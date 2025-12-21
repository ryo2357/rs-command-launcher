use anyhow::Context;
use log::{error, info};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::{Window, WindowBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent};

struct TrayState {
    #[allow(dead_code)]
    tray: tray_icon::TrayIcon,
    #[allow(dead_code)]
    menu: Menu,
    #[allow(dead_code)]
    toggle_item: MenuItem,
    #[allow(dead_code)]
    quit_item: MenuItem,
    toggle_id: MenuId,
    quit_id: MenuId,
}

#[derive(Debug, Clone)]
enum UserEvent {
    TrayIconEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
}

pub fn run_tray() -> anyhow::Result<()> {
    let event_loop = {
        let mut builder = EventLoopBuilder::<UserEvent>::with_user_event();
        builder.build()
    };

    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some({
        let proxy = proxy.clone();
        move |event| {
            let _ = proxy.send_event(UserEvent::TrayIconEvent(event));
        }
    }));

    MenuEvent::set_event_handler(Some({
        let proxy = proxy.clone();
        move |event| {
            let _ = proxy.send_event(UserEvent::MenuEvent(event));
        }
    }));

    let window = WindowBuilder::new()
        .with_title("command-launcher")
        .with_visible(false)
        .build(&event_loop)
        .context("ウィンドウを作成できません")?;

    let mut tray_state: Option<TrayState> = None;
    let mut is_visible = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                if tray_state.is_none() {
                    match build_tray() {
                        Ok(state) => {
                            tray_state = Some(state);
                        }
                        Err(e) => {
                            error!("トレイ初期化に失敗しました: {e:?}");
                        }
                    }
                }
            }
            Event::UserEvent(user_event) => match user_event {
                UserEvent::MenuEvent(ev) => {
                    let id = ev.id;

                    let Some(state) = tray_state.as_ref() else {
                        return;
                    };

                    if state.quit_id == id {
                        info!("終了します");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    if state.toggle_id == id {
                        is_visible = !is_visible;
                        set_window_visible(&window, is_visible);
                    }
                }
                UserEvent::TrayIconEvent(ev) => match ev {
                    TrayIconEvent::Click { .. } | TrayIconEvent::DoubleClick { .. } => {
                        is_visible = !is_visible;
                        set_window_visible(&window, is_visible);
                    }
                    _ => {}
                },
            },
            _ => {}
        }
    });
}

fn set_window_visible(window: &Window, visible: bool) {
    window.set_visible(visible);
    if visible {
        window.request_redraw();
    }
}

fn build_tray() -> anyhow::Result<TrayState> {
    let menu = Menu::new();

    let toggle_item = MenuItem::new("表示/非表示", true, None);
    let quit_item = MenuItem::new("終了", true, None);

    let toggle_id = toggle_item.id().clone();
    let quit_id = quit_item.id().clone();

    menu.append(&toggle_item)
        .context("トレイメニュー項目を追加できません")?;
    menu.append(&quit_item)
        .context("トレイメニュー項目を追加できません")?;

    let icon = make_icon()?;

    let tray = TrayIconBuilder::new()
        // NOTE: tray-icon 側の保持と別に、メニュー/メニュー項目自体も
        // このプロセスが生きている間は保持しておく（drop で項目が消える環境がある）
        .with_menu(Box::new(menu.clone()))
        .with_tooltip("command-launcher")
        .with_icon(icon)
        .build()
        .context("トレイアイコンを作成できません")?;

    Ok(TrayState {
        tray,
        menu,
        toggle_item,
        quit_item,
        toggle_id,
        quit_id,
    })
}

fn make_icon() -> anyhow::Result<Icon> {
    // 最小要件: 外部アセット無しで動く単色アイコン
    let width = 16;
    let height = 16;

    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..(width * height) {
        rgba.extend_from_slice(&[0x20, 0x20, 0x20, 0xFF]);
    }

    Icon::from_rgba(rgba, width, height).context("アイコンを生成できません")
}
