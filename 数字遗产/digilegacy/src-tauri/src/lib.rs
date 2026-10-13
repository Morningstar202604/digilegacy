mod models;
mod db;
mod crypto;
mod platform_data;
mod commands;
mod notify;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            db::init_database(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_all_platforms,
            commands::get_all_platforms_extended,
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
            notify::evaluate_dms,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
