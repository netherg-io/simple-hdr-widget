#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::mem;
use std::time::Duration;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{
    command, Emitter, LogicalSize, Manager, Monitor, PhysicalPosition, Size, WebviewWindow, Wry,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_autostart::ManagerExt;

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

const TARGET_WIDTH_LOGICAL: f64 = 160.0;
const TARGET_HEIGHT_LOGICAL: f64 = 60.0;

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

fn enforce_logical_size(window: &WebviewWindow) {
    if let Ok(current_size) = window.inner_size() {
        let scale_factor = window.scale_factor().unwrap_or(1.0);
        let current_logical_width = current_size.width as f64 / scale_factor;
        let current_logical_height = current_size.height as f64 / scale_factor;

        if (current_logical_width - TARGET_WIDTH_LOGICAL).abs() > 0.1
            || (current_logical_height - TARGET_HEIGHT_LOGICAL).abs() > 0.1
        {
            let _ = window.set_size(Size::Logical(LogicalSize {
                width: TARGET_WIDTH_LOGICAL,
                height: TARGET_HEIGHT_LOGICAL,
            }));
        }
    }
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
    let z_order = if pinned { HWND_TOPMOST } else { HWND_BOTTOM };

    unsafe {
        let _ = SetWindowPos(
            hwnd,
            Some(z_order),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
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
    enforce_logical_size(&window);
}

fn get_current_relative_position(window: &WebviewWindow) -> Option<(f64, f64)> {
    let monitor = window.current_monitor().ok().flatten()?;

    let m_pos = monitor.position();
    let m_size = monitor.size();
    let w_pos = window.outer_position().ok()?;
    let w_size = window.outer_size().ok()?;

    let local_x = (w_pos.x - m_pos.x) as f64;
    let local_y = (w_pos.y - m_pos.y) as f64;

    let max_move_x = (m_size.width as f64 - w_size.width as f64).max(1.0);
    let max_move_y = (m_size.height as f64 - w_size.height as f64).max(1.0);

    let ratio_x = local_x / max_move_x;
    let ratio_y = local_y / max_move_y;

    Some((ratio_x, ratio_y))
}

#[command]
async fn restore_window(
    window: WebviewWindow,
    saved_monitor_name: Option<String>,
    saved_x: Option<f64>,
    saved_y: Option<f64>,
) {
    let monitors = window.available_monitors().unwrap_or_default();
    let mut target_monitor = None;

    if let Some(name) = saved_monitor_name {
        target_monitor = monitors
            .iter()
            .find(|m| m.name().map(|n| n.to_string()).unwrap_or_default() == name)
            .cloned();
    }

    if target_monitor.is_none() {
        target_monitor = window.primary_monitor().ok().flatten();
    }
    if target_monitor.is_none() {
        target_monitor = monitors.first().cloned();
    }

    if let Some(m) = target_monitor {
        let use_percentages = match (saved_x, saved_y) {
            (Some(x), Some(y)) if x.abs() <= 1.0 && y.abs() <= 1.0 => Some((x, y)),
            _ => None,
        };

        if let Some((px, py)) = use_percentages {
            move_window_to_monitor(&window, &m, Some((px, py)));
        } else if let (Some(x), Some(y)) = (saved_x, saved_y) {
            let scale = m.scale_factor();
            let phys_x = (x * scale).round() as i32;
            let phys_y = (y * scale).round() as i32;
            let _ = window.set_position(PhysicalPosition::new(phys_x, phys_y));
        } else {
            move_window_to_monitor(&window, &m, None);
        }
        enforce_logical_size(&window);
    }
}

fn move_window_to_monitor(
    window: &WebviewWindow,
    monitor: &Monitor,
    relative_pos: Option<(f64, f64)>,
) {
    let scale_factor = monitor.scale_factor();
    let m_pos = monitor.position();
    let m_size = monitor.size();

    let target_w_phys = (TARGET_WIDTH_LOGICAL * scale_factor).round();
    let target_h_phys = (TARGET_HEIGHT_LOGICAL * scale_factor).round();

    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
        width: target_w_phys as u32,
        height: target_h_phys as u32,
    }));

    let (rx, ry) = relative_pos.unwrap_or((1.0, 0.0));

    let space_w = m_size.width as f64 - target_w_phys;
    let space_h = m_size.height as f64 - target_h_phys;

    let new_local_x = space_w * rx;
    let new_local_y = space_h * ry;

    let new_x = m_pos.x as f64 + new_local_x;
    let new_y = m_pos.y as f64 + new_local_y;

    let _ = window.set_position(tauri::PhysicalPosition {
        x: new_x.round() as i32,
        y: new_y.round() as i32,
    });
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

    let monitors = window.available_monitors().unwrap_or_default();
    let current_monitor = window.current_monitor().unwrap();

    let current_monitor_name = current_monitor
        .as_ref()
        .and_then(|m| m.name().map(|s| s.to_string()))
        .unwrap_or_default();

    let mut screen_items = vec![];

    for (index, m) in monitors.iter().enumerate() {
        let name = m
            .name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Display {}", index + 1));

        let is_current = name == current_monitor_name;

        let item = CheckMenuItem::with_id(
            &app,
            format!("monitor_select_{}", index),
            format!("Screen {} ({})", index + 1, name),
            true,
            is_current,
            None::<&str>,
        )
        .unwrap();

        screen_items.push(item);
    }

    let screen_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> = screen_items
        .iter()
        .map(|i| i as &dyn tauri::menu::IsMenuItem<Wry>)
        .collect();

    let screens_submenu = Submenu::with_items(&app, "Move to Screen", true, &screen_refs).unwrap();

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
            &screens_submenu,
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
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();

            if id.starts_with("monitor_select_") {
                if let Ok(index) = id.replace("monitor_select_", "").parse::<usize>() {
                    if let Some(window) = app.get_webview_window("main") {
                        if let Ok(monitors) = window.available_monitors() {
                            if let Some(target_monitor) = monitors.get(index) {
                                let rel_pos = get_current_relative_position(&window);
                                move_window_to_monitor(&window, target_monitor, rel_pos);

                                let name = target_monitor
                                    .name()
                                    .map(|s| s.to_string())
                                    .unwrap_or_default();
                                let _ = app.emit("monitor-changed", name);
                            }
                        }
                    }
                }
                return;
            }

            match id {
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
            }
        })
        .setup(|app| {
            let app_handle = app.handle().clone();

            if let Some(window) = app.get_webview_window("main") {
                enforce_logical_size(&window);

                let w_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::ScaleFactorChanged { .. } = event {
                        enforce_logical_size(&w_clone);
                    }
                });

                start_mouse_tracking(window);
            }

            tauri::async_runtime::spawn(async move {
                let mut previous_state = get_hdr_status_internal();

                loop {
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    if let Some(window) = app_handle.get_webview_window("main") {
                        enforce_logical_size(&window);
                    }

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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            toggle_hdr,
            check_hdr_status,
            setup_widget_window,
            set_pin_state,
            move_widget,
            show_context_menu,
            restore_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
