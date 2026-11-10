//! Node 22 under nvm: caveman's CLI and codegraph are npm globals pinned to it, and the MCP
//! registrations use its absolute paths so they never depend on nvm being loaded.

use super::{Component, Report, State};
use crate::ctx::{self, Ctx};
use std::process::Command;

pub const C: Component = Component { name: "node22", deps: &[], optional: false, always_offer: false, detect, apply: |c, _| apply(c) };

fn detect(c: &Ctx) -> Report {
    let nvm = c.nvm_dir.display();
    let r = if let Some(b) = c.node22_bin() {
        let v = ctx::out(Command::new(b.join("node")).arg("--version")).unwrap_or_default();
        Report::new(State::Ok, format!("{} under {nvm}", v.trim()))
    } else if c.nvm_dir.join("nvm.sh").is_file() {
        Report::new(State::Missing, "nvm present, no Node 22")
    } else {
        Report::new(State::Needs, format!("nvm not found at {nvm}; install nvm, then re-run"))
    };
    r.plan([format!(". {nvm}/nvm.sh && nvm install 22")])
}

fn apply(c: &Ctx) -> Result<(), String> {
    // nvm.sh trips over set -u and returns non-zero from harmless checks.
    ctx::run(Command::new("bash").args(["-c", r#"set +eu; . "$NVM_DIR/nvm.sh"; nvm install 22"#]).env("NVM_DIR", &c.nvm_dir));
    c.node22_bin().map(|_| ()).ok_or_else(|| "no Node 22 after nvm install".into())
}
