// The desktop shell is a window around the frontend the Vite server serves.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    genmsi_app_desktop_lib::run()
}
