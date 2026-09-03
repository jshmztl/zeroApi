//! Response 领域模型

use serde::{Deserialize, Serialize};

use crate::domain::{HeaderEntry, Timing};

/// 响应快照
///
/// 与 Request 解耦：一次执行产生一个 ResponseSnapshot，
/// 由 RequestExecution 持有。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSnapshot {
    pub status: u16,
    /// 状态文本（如 "OK"），由传输层从状态码推导
    #[serde(default)]
    pub status_text: String,
    /// 保留重复 Header（如 Set-Cookie）
    pub headers: Vec<HeaderEntry>,
    pub body: ResponseBody,
    pub size_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    pub timing: Timing,
}

/// 响应体
///
/// - Text: 文本响应（受 max_preview_size 限制）
/// - Binary: 二进制 / 超限响应保存为文件
///
/// 序列化为内部标签（与前端 `{type:'text',text}` 契约一致）；
/// 反序列化同时兼容历史数据的外部标签 `{"Text":"…"}` / `{"Binary":{…}}`。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseBody {
    Text { text: String },
    Binary {
        path: String,
        size: u64,
    },
}

impl<'de> Deserialize<'de> for ResponseBody {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        let v = serde_json::Value::deserialize(d).map_err(D::Error::custom)?;
        if let Some(ty) = v.get("type").and_then(|x| x.as_str()) {
            match ty {
                "text" => Ok(ResponseBody::Text {
                    text: v
                        .get("text")
                        .and_then(|x| x.as_str())
                        .unwrap_or_default()
                        .to_string(),
                }),
                "binary" => Ok(ResponseBody::Binary {
                    path: v
                        .get("path")
                        .and_then(|x| x.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    size: v.get("size").and_then(|x| x.as_u64()).unwrap_or(0),
                }),
                _ => Err(D::Error::custom(format!("unknown ResponseBody type: {ty}"))),
            }
        } else if let Some(t) = v.get("Text").and_then(|x| x.as_str()) {
            Ok(ResponseBody::Text { text: t.to_string() })
        } else if let Some(b) = v.get("Binary") {
            Ok(ResponseBody::Binary {
                path: b
                    .get("path")
                    .and_then(|x| x.as_str())
                    .unwrap_or_default()
                    .to_string(),
                size: b.get("size").and_then(|x| x.as_u64()).unwrap_or(0),
            })
        } else {
            Err(D::Error::custom("invalid ResponseBody"))
        }
    }
}

impl ResponseBody {
    pub fn is_binary(&self) -> bool {
        matches!(self, ResponseBody::Binary { .. })
    }
}

/// 文本预览大小上限（默认 5MB，见 Settings.max_preview_size）
pub const DEFAULT_MAX_PREVIEW_SIZE: u64 = 5 * 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_serializes_with_internal_type_tag() {
        let s = serde_json::to_string(&ResponseBody::Text { text: "hello".into() }).unwrap();
        assert_eq!(s, r#"{"type":"text","text":"hello"}"#);
    }

    #[test]
    fn binary_serializes_with_internal_type_tag() {
        let s = serde_json::to_string(&ResponseBody::Binary { path: "a.bin".into(), size: 3 }).unwrap();
        assert_eq!(s, r#"{"type":"binary","path":"a.bin","size":3}"#);
    }

    #[test]
    fn deserializes_both_old_external_and_new_internal_forms() {
        let old_text: ResponseBody = serde_json::from_str(r#"{"Text":"hi"}"#).unwrap();
        assert!(matches!(old_text, ResponseBody::Text { ref text } if text.as_str() == "hi"));
        let old_bin: ResponseBody = serde_json::from_str(r#"{"Binary":{"path":"x.bin","size":5}}"#).unwrap();
        assert!(matches!(old_bin, ResponseBody::Binary { size, .. } if size == 5));

        let new_text: ResponseBody = serde_json::from_str(r#"{"type":"text","text":"yo"}"#).unwrap();
        assert!(matches!(new_text, ResponseBody::Text { ref text } if text.as_str() == "yo"));
        let new_bin: ResponseBody = serde_json::from_str(r#"{"type":"binary","path":"y.bin","size":7}"#).unwrap();
        assert!(matches!(new_bin, ResponseBody::Binary { size, .. } if size == 7));
    }
}
