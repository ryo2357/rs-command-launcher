use std::fs;
use std::path::PathBuf;

use anyhow::Context;
// use log::info;
use serde::Deserialize;

use crate::model::commands::{CommandSpec, Commands, EnvVars};

// 設定のパス

fn app_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

fn config_dir() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().context("home ディレクトリを取得できません")?;
    Ok(home.join(".config").join(app_name()))
}

fn settings_path() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("setting.yaml"))
}

fn local_commands_path() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("local_commands.yaml"))
}

fn env_path() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("env.yaml"))
}
// 読み込み用の書式
#[derive(Debug, Clone, Deserialize)]
struct LoadSettings {
    commands: Vec<CommandSpec>,
}

impl LoadSettings {
    fn inner(self) -> Vec<CommandSpec> {
        self.commands
    }
}

// UIに渡す設定
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    commands: Commands,
}

impl Settings {
    pub fn commands(self) -> Commands {
        self.commands
    }
}

// 読み込み用の書式
#[derive(Debug, Clone, Deserialize)]
struct LocalCommands {
    commands: Vec<CommandSpec>,
}

impl LocalCommands {
    fn inner(self) -> Vec<CommandSpec> {
        self.commands
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoadEnv {
    env: EnvVars,
}

impl LoadEnv {
    fn inner(self) -> EnvVars {
        self.env
    }
}

// 将来的にCommands以外の設定を追加する可能性があるため、この関数名にしている
pub fn load_settings() -> anyhow::Result<Settings> {
    let setting_path = settings_path()?;
    let env_path = env_path()?;
    let local_commands_path = local_commands_path()?;

    let row_settings = load_row_settings(setting_path)?;
    let env_vars = load_env_vars(env_path)?.inner();
    let local_commands = load_local_commands(local_commands_path);
    // info!("local_commands : {:?}", local_commands);

    let commands = row_settings.inner();
    let mut commands = Commands::new(commands);
    // info!("setting : {:?}", commands);
    if let Some(local_cmds) = local_commands {
        commands.extend(Commands::new(local_cmds));
    };
    // info!("local_overay : {:?}", commands);

    // 置換処理

    let commands = commands.expand_vars(env_vars);
    // info!("env_overay : {:?}", commands);

    Ok(Settings { commands })
}

fn load_row_settings(path: PathBuf) -> anyhow::Result<LoadSettings> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("設定ファイルを読み込めません: {}", path.display()))?;
    let row_settings = serde_yaml::from_str::<LoadSettings>(&content)
        .with_context(|| format!("設定ファイルの YAML を解釈できません: {}", path.display()))?;
    Ok(row_settings)
}

fn load_local_commands(path: PathBuf) -> Option<Vec<CommandSpec>> {
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "ローカルコマンドファイルを読み込めません: {}",
            path.display()
        )
    });
    let content = match content {
        Ok(c) => c,
        Err(_) => return None,
    };

    let local_commands = serde_yaml::from_str::<LocalCommands>(&content).with_context(|| {
        format!(
            "ローカルコマンドファイルの YAML を解釈できません: {}",
            path.display()
        )
    });
    let local_commands = match local_commands {
        Ok(c) => c,
        Err(_) => return None,
    };
    Some(local_commands.inner())
}

fn load_env_vars(path: PathBuf) -> anyhow::Result<LoadEnv> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("環境変数ファイルを読み込めません: {}", path.display()))?;
    let env_vars = serde_yaml::from_str::<LoadEnv>(&content).with_context(|| {
        format!(
            "環境変数ファイルの YAML を解釈できません: {}",
            path.display()
        )
    })?;
    Ok(env_vars)
}
