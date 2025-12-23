use std::process::{Child, Command};

use anyhow::Context;

use crate::model::commands::CommandSpec;

pub fn spawn_command(command: &CommandSpec) -> anyhow::Result<Child> {
    let mut cmd = Command::new(command.program());
    cmd.args(command.args());

    cmd.spawn()
        .with_context(|| format!("コマンドを起動できません: {}", command.name()))
}
