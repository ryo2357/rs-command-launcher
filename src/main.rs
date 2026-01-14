// use std::env;

use anyhow::Context;
use log::LevelFilter;
use log::{error, info};
use std::sync::mpsc;

mod app;
mod model;
mod ui;

mod config;
mod runner;

use crate::app::hotkey::Hotkey;
use app::controller::Controller;
use app::endpoint;
use ui::eframe_startup;

fn main() {
    init_logger();

    match start_cli() {
        Ok(_) => {}
        Err(e) => {
            error!("エラー: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn init_logger() {
    // ログ初期化
    // DebugビルドならINFOレベル、ReleaseビルドならWARNログ
    let is_debug = cfg!(debug_assertions);
    env_logger::Builder::new()
        .filter_level(if is_debug {
            LevelFilter::Info
        } else {
            LevelFilter::Warn
        })
        .init();
}

fn start_cli() -> anyhow::Result<()> {
    let settings = config::load_settings()?;

    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("list") => {
            info!("commands list: {:?}", settings.commands());
            return Ok(());
        }
        Some("run-first") => {
            let cmds = settings.commands();
            let first = cmds.first().context("commands が空です")?;

            runner::spawn_command(first)?;
            info!("{:?}を起動しました", first.name());
            return Ok(());
        }
        Some("run") => {
            let name = args.get(2).context("使い方: command-launcher run <name>")?;

            let cmds = settings.commands();
            let cmd = cmds
                .find_by_name(name)
                .with_context(|| format!("指定されたコマンドが見つかりません: {name}"))?;

            runner::spawn_command(cmd)?;
            info!("{:?}を起動しました", cmd.name());
            return Ok(());
        }
        _ => {
            app(settings)?;
        }
    }
    Ok(())
}

fn app(settings: config::Settings) -> anyhow::Result<()> {
    // チャンネル準備
    let (ui_endpoint, ui_handle) = endpoint::create_ui_endpoints();
    let (hotkey_endpoint, hotkey_handle) = endpoint::create_hotkey_endpoints();
    let (tray_endpoint, tray_handle) = endpoint::create_tray_endpoints();
    let (finish_tx, finish_rx) = mpsc::channel::<()>();

    // Controller（司令塔）
    let mut controller = Controller::new(ui_handle, hotkey_handle, tray_handle, finish_rx);
    std::thread::spawn(move || {
        controller.run();
    });

    // タスクトレイ
    // let mut tray = app::task_tray::TaskTray::new(tray_endpoint)?;
    let tray_handle = std::thread::spawn(move || {
        let tray = app::task_tray::TaskTray::new(tray_endpoint);
        match tray.run() {
            Ok(_) => {}
            Err(e) => {
                error!("タスクトレイでエラーが発生しました: {:?}", e);
            }
        };
    });

    // ホットキー
    let mut hotkey = Hotkey::new(hotkey_endpoint)?;
    let hotkey_handle = std::thread::spawn(move || {
        hotkey.run();
    });

    // std::thread::spawn(move || app::hotkey::start(input_tx.clone()));
    // std::thread::spawn(move || app::tray::start(input_tx));

    // UI
    eframe_startup(settings, ui_endpoint)?;
    info!("UI 終了待機中...");

    // 終了処理が完了するのを待つ
    let _ = finish_tx.send(());
    let _ = hotkey_handle.join();
    let _ = tray_handle.join();

    Ok(())
}
