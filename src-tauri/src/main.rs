//! Точка входа Larpscord (Windows).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    larpscord_lib::run()
}
