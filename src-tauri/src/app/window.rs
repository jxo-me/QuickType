use crate::{conf::AppConf, utils};
use tauri::{utils::config::WindowUrl, window::WindowBuilder};

pub fn tray_window(handle: &tauri::AppHandle) {
  let app_conf = AppConf::read();
  let theme = AppConf::theme_mode();
  let app = handle.clone();

  tauri::async_runtime::spawn(async move {
    let link = if app_conf.tray_dashboard {
      "index.html"
    } else {
      &app_conf.tray_origin
    };
    let mut tray_win = WindowBuilder::new(&app, "tray", WindowUrl::App(link.into()))
      .title("QuickType")
      .resizable(false)
      .fullscreen(false)
      .inner_size(app_conf.tray_width, app_conf.tray_height)
      .decorations(false)
      .always_on_top(true)
      .theme(Some(theme))
      .initialization_script(&utils::user_script())
      .initialization_script(include_str!("../scripts/core.js"))
      .user_agent(&app_conf.ua_tray);

    if app_conf.tray_origin == "https://app.quicktype.io" && !app_conf.tray_dashboard {
      tray_win = tray_win
        .initialization_script(include_str!("../vendors/floating-ui-core.js"))
        .initialization_script(include_str!("../vendors/floating-ui-dom.js"))
    }

    tray_win.build().unwrap().hide().unwrap();
  });
}

pub mod cmd {
  use log::info;
  use tauri::{command, Manager};

  #[command]
  pub fn wa_window(
    app: tauri::AppHandle,
    label: String,
    title: String,
    url: String,
    script: Option<String>,
  ) {
    info!("wa_window: {} :=> {}", title, url);
    let win = app.get_window(&label);
    if win.is_none() {
      tauri::async_runtime::spawn(async move {
        tauri::WindowBuilder::new(&app, label, tauri::WindowUrl::App(url.parse().unwrap()))
          .initialization_script(&script.unwrap_or_default())
          .initialization_script(include_str!("../scripts/core.js"))
          .title(title)
          .inner_size(960.0, 700.0)
          .resizable(true)
          .build()
          .unwrap();
      });
    } else if let Some(v) = win {
      if !v.is_visible().unwrap() {
        v.show().unwrap();
      }
      v.eval("window.location.reload()").unwrap();
      v.set_focus().unwrap();
    }
  }

  #[command]
  pub fn window_reload(app: tauri::AppHandle, label: &str) {
    app
      .app_handle()
      .get_window(label)
      .unwrap()
      .eval("window.location.reload()")
      .unwrap();
  }
}
