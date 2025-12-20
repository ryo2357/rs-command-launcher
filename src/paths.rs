use std::path::PathBuf;

use anyhow::Context;

pub fn app_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

pub fn config_dir() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().context("home ディレクトリを取得できません")?;
    Ok(home.join(".config").join(app_name()))
}

pub fn settings_path() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("setting.yaml"))
}

pub fn env_path() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("env.yaml"))
}
