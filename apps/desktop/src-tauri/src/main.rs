#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = bkmt_desktop::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
