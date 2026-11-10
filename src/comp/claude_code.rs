//! Claude Code, native install (~/.local/bin/claude, self-updating).

use super::{Component, Report, State};
use crate::ctx::{self, Ctx};
use std::process::Command;

pub const C: Component = Component { name: "claude-code", deps: &[], optional: false, always_offer: false, detect, apply: |c, _| apply(c) };

const INSTALL: &str = "curl -fsSL https://claude.ai/install.sh | bash";

fn detect(_: &Ctx) -> Report {
    let r = match ctx::which("claude") {
        Some(_) => Report::new(State::Ok, ctx::out(Command::new("claude").arg("--version")).unwrap_or_default().trim()),
        None => Report::new(State::Missing, "not installed"),
    };
    r.plan([INSTALL.to_string()])
}

fn apply(c: &Ctx) -> Result<(), String> {
    if !ctx::run(Command::new("bash").args(["-o", "pipefail", "-c", INSTALL])) {
        return Err("installer failed".into());
    }
    ctx::prepend_path(&c.home.join(".local/bin"));
    ctx::which("claude").ok_or("claude not on PATH after install")?;
    println!("  first run: start `claude` once to log in");
    Ok(())
}
