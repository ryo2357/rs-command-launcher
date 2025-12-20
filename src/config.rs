use std::{collections::BTreeMap, fs, path::Path};

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub commands: Vec<CommandSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommandSpec {
    pub name: String,
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: Option<String>,
}

pub type EnvVars = BTreeMap<String, String>;

pub fn load_settings(path: &Path) -> anyhow::Result<Settings> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("設定ファイルを読み込めません: {}", path.display()))?;
    let settings = serde_yaml::from_str::<Settings>(&content)
        .with_context(|| format!("設定ファイルの YAML を解釈できません: {}", path.display()))?;
    Ok(settings)
}

pub fn load_env_vars(path: &Path) -> anyhow::Result<EnvVars> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("環境変数ファイルを読み込めません: {}", path.display()))?;
    let env_vars = serde_yaml::from_str::<EnvVars>(&content)
        .with_context(|| format!("環境変数ファイルの YAML を解釈できません: {}", path.display()))?;
    Ok(env_vars)
}
