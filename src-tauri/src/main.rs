// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod conf;
mod utils;

use conf::AppConf;
use app::{setup, cmd, fs_extra, menu, window, update};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::{
    fern::colors::{Color, ColoredLevelConfig},
    LogTarget,
};

fn main() {
    let app_conf = AppConf::read().write();

    let context = tauri::generate_context!();

    let mut log = tauri_plugin_log::Builder::default()
    .targets([
      // LogTarget::LogDir,
      // LOG PATH: ~/.quickType/quickType.log
      LogTarget::Folder(utils::app_root()),
      LogTarget::Stdout,
      LogTarget::Webview,
    ])
    .level(log::LevelFilter::Debug);

    if cfg!(debug_assertions) {
        log = log.with_colors(ColoredLevelConfig {
        error: Color::Red,
        warn: Color::Yellow,
        debug: Color::Blue,
        info: Color::BrightGreen,
        trace: Color::Cyan,
        });
    }

    let mut builder = tauri::Builder::default()
        .plugin(log.build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(
          MacosLauncher::LaunchAgent,
          None,
        ))
        .invoke_handler(tauri::generate_handler![
            cmd::drag_window,
            cmd::fullscreen,
            cmd::download,
            cmd::save_file,
            cmd::open_link,
            cmd::run_check_update,
            cmd::open_file,
            cmd::get_data,
            update::download_list,
            update::get_download_list,
            fs_extra::metadata,
            conf::cmd::get_app_conf,
            conf::cmd::reset_app_conf,
            conf::cmd::get_theme,
            conf::cmd::form_confirm,
            conf::cmd::form_cancel,
            conf::cmd::form_msg,
            window::cmd::wa_window,
            window::cmd::window_reload,
        ])
        .setup(setup::init)
        .menu(menu::init());

    if app_conf.tray {
        builder = builder.system_tray(menu::tray_menu());
    }
    if app_conf.save_window_state {
        builder = builder.plugin(tauri_plugin_window_state::Builder::default().build());
    }

    builder
    .on_menu_event(menu::menu_handler)
    .on_system_tray_event(menu::tray_handler)
    .on_window_event(move |event| {
      if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
        let win = event.window().clone();
        let app_conf = AppConf::read();
        if win.label() == "core" {
          if app_conf.isinit {
            tauri::api::dialog::ask(
              Some(event.window()),
              "",
              "Do you want to exit the application when you click the [x] button?",
              move |is_ok| {
                app_conf
                  .amend(serde_json::json!({ "isinit" : false, "main_close": is_ok }))
                  .write();
                if is_ok {
                  std::process::exit(0);
                } else {
                  win.minimize().unwrap();
                }
              },
            );
          } else if app_conf.main_close {
            std::process::exit(0);
          } else {
            win.minimize().unwrap();
          }
        } else {
          event.window().close().unwrap();
        }
        api.prevent_close();
      }
    })
    .run(context)
    .expect("error while running QuickType application");
}
