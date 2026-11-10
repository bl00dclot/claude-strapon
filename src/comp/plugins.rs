//! plugins: user-scope plugins from files/plugins.txt, removing files/plugins-remove.txt.
//! Marketplaces in files/marketplaces.txt are added first. Plugins on neither list are left
//! alone. Extra accounts share them through their plugins/ + settings.json links.

use super::{Component, Report, State};
use crate::ctx::{self, Ctx};
use std::process::Command;

pub const C: Component = Component { name: "plugins", deps: &["claude-code"], optional: false, always_offer: false, detect, apply: |c, _| apply(c) };

fn list(c: &Ctx, file: &str) -> Vec<String> {
    std::fs::read_to_string(c.root.join("files").join(file))
        .unwrap_or_default()
        .lines()
        .map(|l| l.split('#').next().unwrap_or("").trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

struct Diff {
    add: Vec<String>,
    del: Vec<String>,
    off: Vec<String>,
    /// marketplace sources to add
    mkt: Vec<String>,
}

fn claude() -> Command {
    ctx::claude_cmd(None)
}

fn diff(c: &Ctx) -> Result<Diff, String> {
    let (plugins, markets) = std::thread::scope(|s| {
        let m = s.spawn(|| ctx::out(claude().args(["plugin", "marketplace", "list"])));
        (ctx::out(claude().args(["plugin", "list", "--json"])), m.join().ok().flatten())
    });
    let plugins: serde_json::Value =
        serde_json::from_str(&plugins.ok_or("claude plugin list failed")?).map_err(|e| format!("claude plugin list: {e}"))?;
    let user: Vec<(String, bool)> = plugins
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| p["scope"] == "user")
        .filter_map(|p| Some((p["id"].as_str()?.to_string(), p["enabled"].as_bool().unwrap_or(false))))
        .collect();
    let has = |id: &str| user.iter().any(|(i, _)| i == id);
    let want = list(c, "plugins.txt");
    let configured: Vec<String> = markets
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.trim().strip_prefix("❯ ").map(|n| n.trim().to_string()))
        .collect();
    Ok(Diff {
        add: want.iter().filter(|p| !has(p)).cloned().collect(),
        del: list(c, "plugins-remove.txt").into_iter().filter(|p| has(p)).collect(),
        off: want.iter().filter(|p| user.iter().any(|(i, on)| i == *p && !on)).cloned().collect(),
        mkt: list(c, "marketplaces.txt")
            .iter()
            .filter_map(|l| l.split_once(char::is_whitespace))
            .filter(|(name, _)| !configured.iter().any(|n| n == name))
            .map(|(_, src)| src.trim().to_string())
            .collect(),
    })
}

fn short(v: &[String]) -> String {
    v.iter().map(|p| p.split('@').next().unwrap_or(p)).collect::<Vec<_>>().join(" ")
}

fn detect(c: &Ctx) -> Report {
    let d = match diff(c) {
        Ok(d) => d,
        Err(e) => return Report::new(State::Broken, e),
    };
    let mut parts = vec![];
    for (label, v) in [("add marketplace", &d.mkt), ("install", &d.add), ("remove", &d.del), ("enable", &d.off)] {
        if !v.is_empty() {
            parts.push(format!("{label}: {}", short(v)));
        }
    }
    let r = if parts.is_empty() {
        Report::new(State::Ok, format!("{} wanted, all enabled", list(c, "plugins.txt").len()))
    } else {
        Report::new(State::Partial, parts.join(" · "))
    };
    let plan = d
        .mkt
        .iter()
        .map(|m| format!("claude plugin marketplace add {m}"))
        .chain(d.add.iter().map(|p| format!("claude plugin install -s user {p}")))
        .chain(d.off.iter().map(|p| format!("claude plugin enable {p}")))
        .chain(d.del.iter().map(|p| format!("claude plugin uninstall -s user -y {p}")));
    r.plan(plan)
}

fn apply(c: &Ctx) -> Result<(), String> {
    let d = diff(c)?;
    let mut failed = vec![];
    let mut go = |args: &[&str], id: &str| {
        if !ctx::run(claude().args(args).arg(id)) {
            failed.push(id.to_string());
        }
    };
    d.mkt.iter().for_each(|m| go(&["plugin", "marketplace", "add"], m));
    d.add.iter().for_each(|p| go(&["plugin", "install", "-s", "user"], p));
    d.off.iter().for_each(|p| go(&["plugin", "enable"], p));
    d.del.iter().for_each(|p| go(&["plugin", "uninstall", "-s", "user", "-y"], p));
    if failed.is_empty() { Ok(()) } else { Err(format!("failed: {}", failed.join(" "))) }
}
