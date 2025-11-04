use tauri::{
    App, AppHandle, Result, Manager,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
};

/// トレイアイコンをセットアップ
pub fn setup_tray_icon(app: &App) -> Result<()> {
    // メニューを作成
    let show = MenuItem::with_id(app, "show", "ウィンドウを表示", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    // トレイアイコンを作成
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_tray_icon_event)
        .build(app)?;

    Ok(())
}

/// メニュークリックイベントのハンドラー
fn on_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            std::process::exit(0);
        }
        _ => {}
    }
}

/// トレイアイコンクリックイベントのハンドラー
fn on_tray_icon_event(tray: &tauri::tray::TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        if let Some(app) = tray.app_handle().get_webview_window("main") {
            let _ = app.show();
            let _ = app.set_focus();
        }
    }
}
