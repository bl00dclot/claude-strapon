//! codegraph: code-graph MCP server, registered in every logged-in account with absolute
//! node + shim paths: a bare `codegraph` command fails with ENOENT because Claude Code
//! never loads nvm. Tools appear only in repos that ran `codegraph init`. Optional.

use super::{Component, Report, State};
use crate::ctx::{self, Ctx};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const C: Component = Component { name: "codegraph", deps: &["node22", "claude-code"], optional: true, always_offer: false, detect, apply: |c, _| apply(c) };

const PKG: &str = "@colbymchenry/codegraph";

fn shim(c: &Ctx) -> Option<PathBuf> {
    let b = c.node22_bin()?;
    Some(b.parent()?.join("lib/node_modules").join(PKG).join("npm-shim.js"))
}

/// (launcher name, config dir) for account 1 and every logged-in extra account.
fn targets(c: &Ctx) -> Vec<(String, Option<PathBuf>)> {
    let mut v = vec![("claude1".to_string(), None)];
    v.extend(c.accounts().into_iter().filter(|a| a.logged_in()).map(|a| (a.name(), Some(a.dir))));
    v
}

fn detect(c: &Ctx) -> Report {
    let (sh, node) = (shim(c), c.node22_bin().map(|b| b.join("node")));
    let names: Vec<String> = targets(c).into_iter().map(|t| t.0).collect();
    let mut plan = vec![];
    let installed = sh.as_deref().is_some_and(Path::is_file);
    if !installed {
        plan.push(format!("npm i -g {PKG}      (Node 22)"));
    }
    let shown = |p: &Option<PathBuf>| p.as_ref().map_or("<node22>".into(), |p| p.display().to_string());
    plan.push(format!(
        "for {}: claude mcp remove codegraph -s user; claude mcp add codegraph -s user -- {} {} serve --mcp",
        names.join(" "),
        shown(&node),
        shown(&sh)
    ));
    let Some(sh) = sh.filter(|_| installed) else {
        return Report::new(State::Missing, "package not installed under Node 22").plan(plan);
    };
    let v = crate::json::read(&sh.with_file_name("package.json"))
        .and_then(|j| j["version"].as_str().map(String::from))
        .unwrap_or_else(|| "?".into());
    let want = format!("Args: {}", sh.display());
    let bad: Vec<String> = std::thread::scope(|s| {
        let hs: Vec<_> = targets(c)
            .into_iter()
            .map(|(name, dir)| {
                let want = &want;
                s.spawn(move || {
                    let out = ctx::out(ctx::claude_cmd(dir.as_deref()).args(["mcp", "get", "codegraph"])).unwrap_or_default();
                    if out.trim().is_empty() {
                        Some(format!("{name}: not registered"))
                    } else if !out.contains("Connected") {
                        let st = out.lines().find_map(|l| l.trim().strip_prefix("Status: ")).unwrap_or("not connected");
                        Some(format!("{name}: Status: {st}"))
                    } else if !out.contains(want.as_str()) {
                        Some(format!("{name}: not the absolute-path form"))
                    } else {
                        None
                    }
                })
            })
            .collect();
        hs.into_iter().filter_map(|h| h.join().unwrap_or_else(|_| Some("mcp get failed".into()))).collect()
    });
    let r = if bad.is_empty() {
        Report::new(State::Ok, format!("{v} · MCP connected · per repo: codegraph init"))
    } else {
        Report::new(State::Partial, format!("{v} · {}", bad.join("; ")))
    };
    r.plan(plan)
}

fn apply(c: &Ctx) -> Result<(), String> {
    let b = c.node22_bin().ok_or("no Node 22")?;
    let sh = shim(c).ok_or("no Node 22")?;
    if !sh.is_file() {
        let mut path = vec![b.clone()];
        path.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
        let path = std::env::join_paths(path).map_err(|e| e.to_string())?;
        if !ctx::run(Command::new("npm").args(["i", "-g", PKG]).env("PATH", path)) {
            return Err(format!("npm install of {PKG} failed"));
        }
    }
    let node = b.join("node");
    for (name, dir) in targets(c) {
        ctx::ok(ctx::claude_cmd(dir.as_deref()).args(["mcp", "remove", "codegraph", "-s", "user"]));
        let added = ctx::run(
            ctx::claude_cmd(dir.as_deref()).args(["mcp", "add", "codegraph", "-s", "user", "--"]).arg(&node).arg(&sh).args(["serve", "--mcp"]),
        );
        if !added {
            return Err(format!("{name}: claude mcp add codegraph failed"));
        }
    }
    Ok(())
}
