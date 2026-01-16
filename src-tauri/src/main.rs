#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::mem;
use std::time::Duration;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::{command, Emitter, Manager, WebviewWindow};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_positioner::{Position, WindowExt};

use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, DisplayConfigSetDeviceInfo, GetDisplayConfigBufferSizes,
    QueryDisplayConfig, DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_DEVICE_INFO_TYPE,
    DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
    QDC_ONLY_ACTIVE_PATHS,
};
use windows::Win32::Foundation::{ERROR_SUCCESS, HWND, POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetWindowLongW, GetWindowRect, SetWindowLongW, SetWindowPos, GWL_EXSTYLE,
    HWND_BOTTOM, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

const SET_ADVANCED_COLOR_STATE_VAL: i32 = 10;
const ENABLE_FLAG: u32 = 0x1;

#[derive(Clone, serde::Serialize)]
struct HdrStatusPayload {
    enabled: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DisplayConfigSetAdvancedColorState {
    pub header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
    pub value: u32,
}

fn get_active_paths() -> Result<Vec<DISPLAYCONFIG_PATH_INFO>, String> {
    let mut path_count = 0;
    let mut mode_count = 0;

    let result = unsafe {
        GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count)
    };

    if result != ERROR_SUCCESS {
        return Err(format!("GetDisplayConfigBufferSizes failed: {:?}", result));
    }

    let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
    let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];

    let result = unsafe {
        QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            None,
        )
    };

    if result != ERROR_SUCCESS {
        return Err(format!("QueryDisplayConfig failed: {:?}", result));
    }

    paths.truncate(path_count as usize);
    Ok(paths)
}

fn get_hdr_status_internal() -> bool {
    if let Ok(paths) = get_active_paths() {
        for path in paths {
            let mut get_packet = DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO,
                    size: mem::size_of::<DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO>() as u32,
                    adapterId: path.targetInfo.adapterId,
                    id: path.targetInfo.id,
                },
                ..Default::default()
            };

            let ret =
                unsafe { DisplayConfigGetDeviceInfo(&mut get_packet.header as *mut _ as *mut _) };

            if ret == ERROR_SUCCESS.0 as i32 {
                let enabled = unsafe { (get_packet.Anonymous.value & 0x2) != 0 };
                if enabled {
                    return true;
                }
            }
        }
    }
    false
}

fn start_mouse_tracking(window: WebviewWindow) {
    std::thread::spawn(move || {
        let tauri_hwnd = window.hwnd().expect("Failed to get HWND");

        let hwnd = HWND(tauri_hwnd.0 as *mut _);

        let mut was_inside = false;

        loop {
            std::thread::sleep(Duration::from_millis(50));

            unsafe {
                let mut cursor_pos = POINT::default();
                let mut win_rect = RECT::default();

                if GetCursorPos(&mut cursor_pos).is_err() {
                    continue;
                }
                if GetWindowRect(hwnd, &mut win_rect).is_err() {
                    continue;
                }

                let is_inside = cursor_pos.x >= win_rect.left
                    && cursor_pos.x <= win_rect.right
                    && cursor_pos.y >= win_rect.top
                    && cursor_pos.y <= win_rect.bottom;

                if is_inside != was_inside {
                    was_inside = is_inside;
                    if !is_inside {
                        let _ = window.emit("mouse-left-window", ());
                    }
                }
            }
        }
    });
}

#[command]
async fn toggle_hdr(enable: bool) -> Result<String, String> {
    let paths = get_active_paths()?;
    let mut log = String::new();
    let mut success_any = false;

    for path in paths {
        let mut set_packet = DisplayConfigSetAdvancedColorState {
            header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                r#type: DISPLAYCONFIG_DEVICE_INFO_TYPE(SET_ADVANCED_COLOR_STATE_VAL),
                size: mem::size_of::<DisplayConfigSetAdvancedColorState>() as u32,
                adapterId: path.targetInfo.adapterId,
                id: path.targetInfo.id,
            },
            value: if enable { ENABLE_FLAG } else { 0 },
        };

        let ret = unsafe { DisplayConfigSetDeviceInfo(&mut set_packet.header as *mut _ as *mut _) };

        if ret == ERROR_SUCCESS.0 as i32 {
            success_any = true;
            log.push_str(&format!("Success for ID {}\n", path.targetInfo.id));
        } else {
            log.push_str(&format!("Error {} for ID {}\n", ret, path.targetInfo.id));
        }
    }

    if success_any {
        Ok(log)
    } else {
        Err(log)
    }
}

#[command]
async fn check_hdr_status() -> Result<bool, String> {
    Ok(get_hdr_status_internal())
}

#[command]
async fn setup_widget_window(window: WebviewWindow) {
    let tauri_hwnd = window.hwnd().expect("Failed to get HWND");
    let hwnd = HWND(tauri_hwnd.0 as *mut _);

    unsafe {
        let mut style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        style &= !WS_EX_APPWINDOW.0 as i32;
        style |= WS_EX_TOOLWINDOW.0 as i32;
        SetWindowLongW(hwnd, GWL_EXSTYLE, style);

        let _ = SetWindowPos(
            hwnd,
            Some(HWND_BOTTOM),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

#[command]
async fn set_pin_state(window: WebviewWindow, pinned: bool) {
    let tauri_hwnd = window.hwnd().expect("Failed to get HWND");
    let hwnd = HWND(tauri_hwnd.0 as *mut _);

    unsafe {
        if pinned {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        } else {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_BOTTOM),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }
}

#[command]
async fn move_widget(window: WebviewWindow, x: i32, y: i32, is_pinned: bool) {
    let tauri_hwnd = window.hwnd().expect("Failed to get HWND");
    let hwnd = HWND(tauri_hwnd.0 as *mut _);

    unsafe {
        let z_order = if is_pinned { None } else { Some(HWND_BOTTOM) };
        let flags = if is_pinned {
            SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER
        } else {
            SWP_NOSIZE | SWP_NOACTIVATE
        };

        let _ = SetWindowPos(hwnd, z_order, x, y, 0, 0, flags);
    }
}

#[command]
async fn init_position(window: WebviewWindow) {
    let _ = window.as_ref().window().move_window(Position::BottomRight);
}

#[command]
async fn show_context_menu(
    app: tauri::AppHandle,
    window: WebviewWindow,
    is_pinned: bool,
    is_draggable: bool,
) {
    let autostart_manager = app.autolaunch();
    let is_autostart = autostart_manager.is_enabled().unwrap_or(false);

    let toggle_pin = CheckMenuItem::with_id(
        &app,
        "toggle_pin",
        "Always on Top",
        true,
        is_pinned,
        None::<&str>,
    )
    .unwrap();

    let toggle_drag = CheckMenuItem::with_id(
        &app,
        "toggle_drag",
        "Allow Drag",
        true,
        is_draggable,
        None::<&str>,
    )
    .unwrap();

    let separator = PredefinedMenuItem::separator(&app).unwrap();

    let toggle_autostart = CheckMenuItem::with_id(
        &app,
        "toggle_autostart",
        "Run on Startup",
        true,
        is_autostart,
        None::<&str>,
    )
    .unwrap();

    let kill_app = MenuItem::with_id(&app, "kill_app", "Kill Widget", true, None::<&str>).unwrap();

    let menu = Menu::with_items(
        &app,
        &[
            &toggle_pin,
            &toggle_drag,
            &separator,
            &toggle_autostart,
            &kill_app,
        ],
    )
    .unwrap();

    let _ = window.popup_menu(&menu);
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle_pin" => {
                let _ = app.emit("menu-toggle-pin", ());
            }
            "toggle_drag" => {
                let _ = app.emit("menu-toggle-drag", ());
            }
            "toggle_autostart" => {
                let manager = app.autolaunch();
                if manager.is_enabled().unwrap_or(false) {
                    let _ = manager.disable();
                } else {
                    let _ = manager.enable();
                }
            }
            "kill_app" => app.exit(0),
            _ => {}
        })
        .setup(|app| {
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let mut previous_state = get_hdr_status_internal();

                loop {
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    let current_state = get_hdr_status_internal();

                    if current_state != previous_state {
                        previous_state = current_state;
                        println!("HDR State changed to: {}", current_state);

                        let _ = app_handle.emit(
                            "hdr-state-changed",
                            HdrStatusPayload {
                                enabled: current_state,
                            },
                        );
                    }
                }
            });

            if let Some(window) = app.get_webview_window("main") {
                start_mouse_tracking(window);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            toggle_hdr,
            check_hdr_status,
            init_position,
            setup_widget_window, // Убедись, что все твои команды тут
            set_pin_state,
            move_widget,
            show_context_menu
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
