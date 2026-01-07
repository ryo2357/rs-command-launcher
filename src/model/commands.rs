use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::HashSet;

pub type EnvVars = BTreeMap<String, String>;

#[derive(Debug, Clone, Deserialize)]
pub struct CommandSpec {
    name: String,
    program: String,
    #[serde(default)]
    args: Vec<String>,
}

impl CommandSpec {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn program(&self) -> &str {
        &self.program
    }
    pub fn args(&self) -> &Vec<String> {
        &self.args
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Commands {
    inner: Vec<CommandSpec>,
}

impl Commands {
    pub fn new(inner: Vec<CommandSpec>) -> Self {
        let mut cmds = Commands { inner };
        cmds.dedup_by_name();
        cmds
    }
    pub fn first(&self) -> Option<&CommandSpec> {
        self.inner.first()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&CommandSpec> {
        self.inner.iter().find(|cmd| cmd.name == name)
    }

    // CommandsとCommandsの結合
    pub fn extend(&mut self, other: Commands) {
        self.inner.extend(other.inner);
        self.dedup_by_name_keep_last();
    }

    // nameの重複削除メソッド（最初の出現を残す）
    fn dedup_by_name(&mut self) {
        let mut seen = HashSet::new();
        // retain はクロージャ―がtrueを返す要素だけを残す
        self.inner.retain(|cmd| seen.insert(cmd.name.clone()));
    }

    // nameの重複削除メソッド（最後の出現を残す）
    fn dedup_by_name_keep_last(&mut self) {
        // ベクタを反転して最初の出現を残し、再び反転することで
        // 元の順序を保ちつつ最後の出現を残す
        self.inner.reverse();
        let mut seen = HashSet::new();
        self.inner.retain(|cmd| seen.insert(cmd.name.clone()));
        self.inner.reverse();
    }

    // 環境変数による置換処理
    pub fn expand_vars(self, env: EnvVars) -> Self {
        let new_inner: Vec<CommandSpec> = self
            .inner
            .into_iter()
            .map(|cmd| CommandSpec {
                name: cmd.name.clone(),
                program: expand_var_in_string(cmd.program, &env),
                args: cmd
                    .args
                    .iter()
                    .map(|arg| expand_var_in_string(arg.clone(), &env))
                    .collect(),
            })
            .collect();

        Commands { inner: new_inner }
    }
}

// 文字列中の $name を置換。未定義はそのまま残す。
pub fn expand_var_in_string(s: String, env: &EnvVars) -> String {
    if s.is_empty() {
        return s;
    }

    if let Some(rest) = s.strip_prefix('$') {
        let name: String = rest.to_string();
        if name.is_empty() {
            return "$".to_string();
        }
        if let Some(value) = env.get(&name) {
            return value.to_string();
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_by_name_removes_duplicates() {
        let mut cmds = Commands {
            inner: vec![
                CommandSpec {
                    name: "a".into(),
                    program: "p1".into(),
                    args: vec![],
                },
                CommandSpec {
                    name: "b".into(),
                    program: "p2".into(),
                    args: vec![],
                },
                CommandSpec {
                    name: "a".into(),
                    program: "p3".into(),
                    args: vec![],
                },
            ],
        };

        cmds.dedup_by_name();
        assert_eq!(cmds.inner.len(), 2);
        assert_eq!(cmds.inner[0].name, "a");
        assert_eq!(cmds.inner[1].name, "b");
        println!("cmd:{:?}", cmds);
    }
}
