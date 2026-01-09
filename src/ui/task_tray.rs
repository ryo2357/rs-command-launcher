use eframe::egui;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, OnceLock, mpsc};
use std::thread;
use std::time::Duration;

use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_NULL};

#[derive(Debug, Clone, Copy)]
pub enum TrayCommand {
    Show,
    Quit,
}

pub struct TaskTray {
    _tray: TrayIcon,
    _menu: Menu,
    _item_show: MenuItem,
    _item_quit: MenuItem,

    hwnd_ref: Arc<AtomicIsize>,
    ctx_ref: Arc<OnceLock<egui::Context>>,
}

impl TaskTray {
    pub fn new() -> anyhow::Result<(Self, mpsc::Receiver<TrayCommand>)> {
        let (tx, rx) = mpsc::channel::<TrayCommand>();

        let hwnd_ref = Arc::new(AtomicIsize::new(0));
        let hwnd_ref_thread = Arc::clone(&hwnd_ref);

        let ctx_ref: Arc<OnceLock<egui::Context>> = Arc::new(OnceLock::new());
        let ctx_ref_thread = Arc::clone(&ctx_ref);

        let menu = Menu::new();
        let item_show = MenuItem::new("表示", true, None);
        let item_quit = MenuItem::new("終了", true, None);
        menu.append_items(&[&item_show, &item_quit])?;

        let show_id = item_show.id().clone();
        let quit_id = item_quit.id().clone();

        let icon = load_tray_icon_from_embedded_png()?;

        let tray = TrayIconBuilder::new()
            .with_tooltip("rs-command-launcher")
            .with_menu(Box::new(menu.clone()))
            .with_icon(icon)
            .build()?;

        let menu_rx = MenuEvent::receiver();

        thread::Builder::new()
            .name("task-tray-event-loop".to_string())
            .spawn(move || {
                loop {
                    if let Ok(event) = menu_rx.try_recv() {
                        let cmd = if event.id == show_id {
                            Some(TrayCommand::Show)
                        } else if event.id == quit_id {
                            Some(TrayCommand::Quit)
                        } else {
                            None
                        };

                        if let Some(cmd) = cmd {
                            if tx.send(cmd).is_err() {
                                break;
                            }

                            // eframe/egui の update を確実に起こす
                            if let Some(ctx) = ctx_ref_thread.get() {
                                ctx.request_repaint();
                            }

                            // 念のため Win32 側も起こす
                            let hwnd = hwnd_ref_thread.load(Ordering::Relaxed);
                            if hwnd != 0 {
                                unsafe { PostMessageW(hwnd as HWND, WM_NULL, 0, 0) };
                            }
                        }
                    }

                    thread::sleep(Duration::from_millis(16));
                }
            })?;

        Ok((
            Self {
                _tray: tray,
                _menu: menu,
                _item_show: item_show,
                _item_quit: item_quit,
                hwnd_ref,
                ctx_ref,
            },
            rx,
        ))
    }

    pub fn set_hwnd(&self, hwnd: HWND) {
        self.hwnd_ref.store(hwnd, Ordering::Relaxed);
    }

    pub fn set_ctx(&self, ctx: egui::Context) {
        // 1回だけ設定されればOK（既に設定済みなら無視）
        let _ = self.ctx_ref.set(ctx);
    }
}

fn load_tray_icon_from_embedded_png() -> anyhow::Result<Icon> {
    // 推奨: 32x32 透過PNG
    // 例: e:\dev\rs-command-launcher\assets\tray.png を用意
    let bytes: &[u8] = include_bytes!("../../assets/tray.png");

    let img = image::load_from_memory(bytes)?.into_rgba8();
    let (w, h) = img.dimensions();
    let rgba = img.into_raw();

    Ok(Icon::from_rgba(rgba, w, h)?)
}
