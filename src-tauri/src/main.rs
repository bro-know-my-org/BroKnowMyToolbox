// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// 在 Windows 发布版中防止出现额外的控制台窗口，切勿删除！！

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    bro_know_my_toolbox_lib::run()
}
