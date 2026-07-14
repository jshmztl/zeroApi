// 防止 Windows 控制台在发布版本中弹出
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    zeroapi_lib::run()
}
