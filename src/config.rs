use std::fs;
use std::path::PathBuf;

use anyhow::Context;
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

fn env_path() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("env.yaml"))
}
// 読み込み用の書式
#[derive(Debug, Clone, Deserialize)]
pub struct LoadSettings {
    commands: Vec<CommandSpec>,
}

impl LoadSettings {
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
pub fn load_settings() -> anyhow::Result<Commands> {
    let setting_path = settings_path()?;
    let env_path = env_path()?;

    let row_settings = load_row_settings(setting_path)?;
    let env_vars = load_env_vars(env_path)?.inner();

    let commands = row_settings.inner();
    let commands = Commands::new(commands);

    // 置換処理

    let commands= commands.expand_vars(env_vars);

    Ok(commands)
}

fn load_row_settings(path: PathBuf) -> anyhow::Result<LoadSettings> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("設定ファイルを読み込めません: {}", path.display()))?;
    let row_settings = serde_yaml::from_str::<LoadSettings>(&content)
        .with_context(|| format!("設定ファイルの YAML を解釈できません: {}", path.display()))?;
    Ok(row_settings)
}

fn load_env_vars(path: PathBuf) -> anyhow::Result<LoadEnv> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("環境変数ファイルを読み込めません: {}", path.display()))?;
    let env_vars = serde_yaml::from_str::<LoadEnv>(&content)
        .with_context(|| format!("環境変数ファイルの YAML を解釈できません: {}", path.display()))?;
    Ok(env_vars)
}

