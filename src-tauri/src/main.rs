#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::os::windows::process::CommandExt;
use std::process::Command;
use tauri::{command, Window};
use tauri_plugin_positioner::{Position, WindowExt};

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[command]
fn toggle_hdr() -> Result<String, String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;

    enigo.key(Key::Meta, Direction::Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Alt, Direction::Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Unicode('b'), Direction::Click).map_err(|e| e.to_string())?;
    enigo.key(Key::Alt, Direction::Release).map_err(|e| e.to_string())?;
    enigo.key(Key::Meta, Direction::Release).map_err(|e| e.to_string())?;

    Ok("HDR Toggle command sent".into())
}

#[command]
async fn check_hdr_status() -> Result<bool, String> {
    let output = Command::new("powershell")
        .args(&[
            "-NoProfile",
            "-Command",
            r#"
            # Get HDR status from registry (User GameConfigStore)
            $status = (Get-ItemProperty HKCU:\System\GameConfigStore -Name "HDREnabled" -ErrorAction SilentlyContinue).HDREnabled
            if ($status -eq 1) { Write-Output "On" } else { Write-Output "Off" }
            "#,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(result == "On")
}

#[command]
async fn init_position(window: Window) {
    let _ = window.set_shadow(false);
    let _ = window.move_window(Position::TopRight);
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .invoke_handler(tauri::generate_handler![
            toggle_hdr,
            check_hdr_status,
            init_position
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
