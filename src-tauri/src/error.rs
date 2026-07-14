//! 统一错误类型
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("URL 错误: {0}")]
    Url(#[from] url::ParseError),

    #[error("HTTP 错误: {0}")]
    Http(#[from] reqwest::Error),

    #[error("请求头错误: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),

    #[error("cURL 解析错误: {0}")]
    Curl(String),

    #[error("{0}")]
    Other(String),
}

pub type AppResult<T> = Result<T, AppError>;

// 让 Tauri 能将 AppError 序列化为前端可读错误
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
