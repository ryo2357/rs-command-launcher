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
}

pub type EnvVars = BTreeMap<String, String>;

pub fn load_settings(setting_path: &Path,env_path: &Path) -> anyhow::Result<Settings> {
    let row_settings = load_row_settings(setting_path)?;
    let env_vars = load_env_vars(env_path)?;

    // 置換処理
    let expanded = expand_settings(row_settings, &env_vars);

    Ok(expanded)
}

fn load_row_settings(path: &Path) -> anyhow::Result<Settings> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("設定ファイルを読み込めません: {}", path.display()))?;
    let row_settings = serde_yaml::from_str::<Settings>(&content)
        .with_context(|| format!("設定ファイルの YAML を解釈できません: {}", path.display()))?;
    Ok(row_settings)
}

fn load_env_vars(path: &Path) -> anyhow::Result<EnvVars> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("環境変数ファイルを読み込めません: {}", path.display()))?;
    let env_vars = serde_yaml::from_str::<EnvVars>(&content)
        .with_context(|| format!("環境変数ファイルの YAML を解釈できません: {}", path.display()))?;
    Ok(env_vars)
}

fn expand_settings(settings: Settings, env: &EnvVars) -> Settings {

    let new_settings = Settings {
        commands: settings.commands.into_iter().map(|cmd| {
            CommandSpec {
                name: cmd.name,
                program: expand_var_in_string(cmd.program, env),
                args: cmd.args.into_iter()
                  .map(|arg| expand_var_in_string(arg, env)).collect(),
            }
        }).collect(),
    };

    new_settings
}


// 文字列中の $name を置換。未定義はそのまま残す。
pub fn expand_var_in_string(s: String, env: &EnvVars) -> String {
    if s.is_empty() {
        return s;
    } 

    if let Some(rest) = s.strip_prefix('$') {
        let name:String = rest.to_string();
        if name.is_empty() {
            return "$".to_string();
        }
        if let Some(value) = env.get(&name) {
            return value.to_string();
        }
    } 
    s

}


