mod commands;
mod crypto;
mod db;
mod heartbeat;
mod models;
mod notify;
mod platform_data;
mod recovery;
mod vault;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

/// 后台推进线程：每 30 分钟评估 DMS（写发件箱）并尝试云端心跳
fn spawn_background_loop(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        if let Err(e) = notify::evaluate_dms(app_handle.clone()) {
            eprintln!("后台 DMS 评估失败: {}", e);
        }
        heartbeat::send_heartbeat();
        std::thread::sleep(std::time::Duration::from_secs(30 * 60));
    });
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let checkin = MenuItem::with_id(app, "checkin", "我还安好（安全确认）", true, None::<&str>)?;
    let lock_item = MenuItem::with_id(app, "lock", "锁定应用", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &checkin, &lock_item, &quit])?;

    let mut tray = TrayIconBuilder::with_id("digilegacy-tray")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("归零 · 数字遗产管理系统")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.unminimize();
                    let _ = win.set_focus();
                }
            }
            "checkin" => {
                let _ = db::record_check_in_db();
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.emit("dms-checked-in", ());
                }
            }
            "lock" => {
                crate::vault::clear_key();
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.emit("app-locked", ());
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(win) = tray.app_handle().get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.unminimize();
                    let _ = win.set_focus();
                }
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }
    tray.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            db::init_database(app.handle())?;
            build_tray(app)?;
            spawn_background_loop(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_all_platforms,
            commands::get_all_platforms_extended,
            commands::import_platform_patch,
            commands::export_platform_patch,
            commands::platform_patch_status,
            commands::clear_platform_patch,
            commands::add_asset,
            commands::update_asset,
            commands::delete_asset,
            commands::get_all_assets,
            commands::get_asset_by_id,
            commands::get_assets_by_category,
            commands::set_disposal_wish,
            commands::set_user_profile,
            commands::get_user_profile,
            commands::get_inheritance_report,
            commands::export_report_to_file,
            commands::export_zip_to_file,
            commands::update_check_in,
            commands::get_check_in_status,
            commands::set_dead_mans_switch,
            commands::get_dashboard_stats,
            commands::security_status,
            commands::set_master_password,
            commands::unlock,
            commands::lock,
            commands::set_asset_credential,
            commands::get_asset_credential,
            commands::change_master_password,
            commands::export_backup,
            commands::import_backup,
            commands::open_external,
            commands::start_recovery,
            commands::complete_recovery,
            commands::get_recovery_seal_exists,
            commands::get_heartbeat_status,
            commands::get_heartbeat_config,
            commands::set_heartbeat_config,
            commands::send_heartbeat_now,
            commands::set_autostart,
            commands::hide_main_window,
            notify::evaluate_dms,
        ])
        .on_window_event(|window, event| {
            // 关闭窗口 = 隐藏到托盘（驻留后台推进提醒与心跳）
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
