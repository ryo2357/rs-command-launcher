use std::process::{Child, Command};

use anyhow::Context;

use crate::config::{CommandSpec, EnvVars};

pub fn spawn_command(command: &CommandSpec, env_vars: &EnvVars) -> anyhow::Result<Child> {
    let mut cmd = Command::new(&command.program);
    cmd.args(&command.args);
    cmd.envs(env_vars);

    if let Some(cwd) = &command.cwd {
        cmd.current_dir(cwd);
    }

    cmd.spawn()
        .with_context(|| format!("コマンドを起動できません: {}", command.name))
}
