use log::info;
use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};
use windows_sys::Win32::{
    System::Com::CoUninitialize,
    UI::WindowsAndMessaging::{DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage},
};

use crate::app::endpoint::{TrayCmd, TrayEndpoint, TrayEvent};

// 推奨: 32x32 透過PNG
// build.rs でコンパイル時に埋め込み
// 例: e:\dev\rs-command-launcher\assets\tray.png を用意
include!(concat!(env!("OUT_DIR"), "/tray_icon_data.rs"));
fn load_tray_icon_from_embedded_png() -> anyhow::Result<Icon> {
    // コンパイル時に埋め込まれた RGBA データを使用して、実行時に Icon ハンドルを生成
    Ok(Icon::from_rgba(
        TRAY_RGBA.to_vec(),
        TRAY_WIDTH,
        TRAY_HEIGHT,
    )?)
}

pub struct TaskTray {
    endpoint: TrayEndpoint,
}

impl TaskTray {
    pub fn new(endpoint: TrayEndpoint) -> Self {
        Self { endpoint }
    }

    pub fn run(self) -> anyhow::Result<()> {
        let icon = load_tray_icon_from_embedded_png()?;
        let menu = Menu::new();
        let item_show = MenuItem::new("Show Window", true, None);
        let item_quit = MenuItem::new("Quit", true, None);
        let _ = menu.append_items(&[&item_show, &item_quit]);

        let show_id = item_show.id().clone();
        let quit_id = item_quit.id().clone();

        let _tray = TrayIconBuilder::new()
            .with_tooltip("rs-command-launcher")
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .build()?;

        let menu_rx = MenuEvent::receiver();
        // let tray_rx = TrayIconEvent::receiver();

        loop {
            // 重要: まず Win32 のメッセージを捌く（右クリック等が動くようになる）
            pump_win32_messages_once();

            // メニューイベントの処理
            if let Ok(event) = menu_rx.try_recv() {
                info!("Menu event received: {:?}", event);
                if event.id == show_id {
                    // info!("Show Window menu item clicked");
                    let _ = self.endpoint.tx.send(TrayEvent::ShowWindow);
                } else if event.id == quit_id {
                    // info!("Quit menu item clicked");
                    let _ = self.endpoint.tx.send(TrayEvent::Quit);
                }
            }
            // トレイイベント（必要なら）
            // 　トレイアイコンに触れた際などに発火される
            // if let Ok(event) = tray_rx.try_recv() {
            //     info!("Tray event received: {:?}", event);
            // }

            // コントローラーからの受信処理
            if let Ok(cmd) = self.endpoint.rx.try_recv() {
                match cmd {
                    TrayCmd::Finish => {
                        info!("コントローラーからの終了処理受信");
                        break;
                    }
                }
            }
            // CPU負荷を抑えるために短いスリープを挿入
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        unsafe {
            CoUninitialize();
        }
        Ok(())
    }
}

fn pump_win32_messages_once() {
    // このスレッドに配送されている Win32 メッセージを捌く
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while PeekMessageW(&mut msg as *mut MSG, 0, 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg as *const MSG);
            DispatchMessageW(&msg as *const MSG);
        }
    }
}
