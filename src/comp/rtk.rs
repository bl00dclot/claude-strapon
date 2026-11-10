//! rtk (Rust Token Killer): PreToolUse(Bash) hook that compresses command output.
//! Another project is also called rtk (Rust Type Kit); `rtk gain` tells them apart.

use super::{Component, Report, State};
use crate::ctx::{self, Ctx};
use std::process::Command;

pub const C: Component = Component { name: "rtk", deps: &["claude-code"], optional: true, always_offer: false, detect, apply: |c, _| apply(c) };

const INSTALL: &str = "curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/master/install.sh | sh";

fn detect(c: &Ctx) -> Report {
    let init = format!("rtk init -g --auto-patch      (hook + RTK.md + @RTK.md in {})", c.claude_dir.display());
    if ctx::which("rtk").is_none() {
        return Report::new(State::Missing, "not installed").plan([INSTALL.to_string(), init]);
    }
    let (gain, show, ver) = std::thread::scope(|s| {
        let gain = s.spawn(|| ctx::ok(Command::new("rtk").arg("gain")));
        let show = s.spawn(|| ctx::out(Command::new("rtk").args(["init", "--show"])).unwrap_or_default());
        let ver = ctx::out(Command::new("rtk").arg("--version")).unwrap_or_default();
        (gain.join().unwrap_or(false), show.join().unwrap_or_default(), ver)
    });
    let r = if !gain {
        Report::new(State::Broken, "`rtk` on PATH is not Rust Token Killer (rtk gain fails)")
    } else {
        let v = ver.split_whitespace().nth(1).unwrap_or("?");
        let wired = ["[ok] Hook", "[ok] settings.json"].iter().all(|p| show.lines().any(|l| l.starts_with(p)));
        if wired { Report::new(State::Ok, format!("{v} · hook ok")) } else { Report::new(State::Partial, format!("{v} · hook not wired")) }
    };
    r.plan([init])
}

fn apply(c: &Ctx) -> Result<(), String> {
    if ctx::which("rtk").is_none() {
        if !ctx::run(Command::new("bash").args(["-o", "pipefail", "-c", INSTALL])) {
            return Err("rtk installer failed".into());
        }
        ctx::prepend_path(&c.home.join(".local/bin"));
    }
    ctx::run(Command::new("rtk").args(["init", "-g", "--auto-patch"])).then_some(()).ok_or_else(|| "rtk init failed".into())
}
