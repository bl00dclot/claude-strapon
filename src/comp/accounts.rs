//! accounts: every extra Claude account (~/.claude-accountN, N ≥ 2) shares account 1's
//! settings, hooks, skills and plugins by symlink. Its .claude.json (login + MCP servers)
//! cannot be a link, so MCP servers it lacks are copied in from account 1 (its own entries
//! win). Launchers come from files/claude-accounts.sh, sourced from ~/.bashrc.

use super::{Component, Opts, Report, Row, State};
use crate::ctx::{Account, Ctx};
use crate::json;
use serde_json::{Map, Value};
use std::fs;
use std::os::unix::fs::{DirBuilderExt, symlink};
use std::path::Path;

pub const C: Component = Component { name: "accounts", deps: &[], optional: false, always_offer: true, detect, apply };

pub const LINKS: [&str; 7] = ["settings.json", "settings.local.json", "CLAUDE.md", "RTK.md", "agents", "skills", "plugins"];

fn exists(p: &Path) -> bool {
    fs::symlink_metadata(p).is_ok()
}

fn launcher_line(c: &Ctx) -> String {
    format!(". \"{}\"", c.root.join("files/claude-accounts.sh").display())
}

/// The accounts line, or the caveman route line (claude-route.sh sources the accounts file).
fn launchers_wired(c: &Ctx) -> bool {
    let route = format!(". \"{}\"", c.root.join("files/claude-route.sh").display());
    let want = launcher_line(c);
    fs::read_to_string(&c.bashrc).unwrap_or_default().lines().any(|l| l.trim() == want || l.trim() == route)
}

fn servers(p: &Path) -> Map<String, Value> {
    json::read(p).and_then(|v| v["mcpServers"].as_object().cloned()).unwrap_or_default()
}

fn missing_mcp(c: &Ctx, a: &Account) -> Vec<String> {
    let have = servers(&a.dir.join(".claude.json"));
    servers(&c.claude_json).keys().filter(|k| !have.contains_key(*k)).cloned().collect()
}

struct Links {
    ok: usize,
    conflict: Vec<&'static str>,
    absent: Vec<&'static str>,
}

fn links(c: &Ctx, a: &Account) -> Links {
    let mut l = Links { ok: 0, conflict: vec![], absent: vec![] };
    for f in LINKS {
        let p = a.dir.join(f);
        if fs::read_link(&p).is_ok_and(|t| t == c.claude_dir.join(f)) {
            l.ok += 1;
        } else if exists(&p) {
            l.conflict.push(f);
        } else {
            l.absent.push(f);
        }
    }
    l
}

fn row(c: &Ctx, a: &Account) -> (Row, Vec<String>) {
    let l = links(c, a);
    let miss = if a.logged_in() { missing_mcp(c, a) } else { vec![] };
    let mut plan: Vec<String> = l
        .absent
        .iter()
        .map(|f| format!("ln -s {} {}", c.claude_dir.join(f).display(), a.dir.join(f).display()))
        .collect();
    if !miss.is_empty() {
        plan.push(format!("copy MCP servers into {}: {}", a.dir.join(".claude.json").display(), miss.join(" ")));
    }
    if !a.logged_in() {
        plan.push(format!("then by hand: {}   (log in), re-run", a.name()));
    }
    let mcp = if miss.is_empty() { String::new() } else { format!(" · MCP missing: {}", miss.join(" ")) };
    let (state, detail) = if !l.conflict.is_empty() {
        (State::Broken, format!("not links, move aside by hand: {}", l.conflict.join(" ")))
    } else if !l.absent.is_empty() {
        let login = if a.logged_in() { String::new() } else { format!(" · then log in: {}", a.name()) };
        (State::Partial, format!("{}/{} links{mcp}{login}", l.ok, LINKS.len()))
    } else if !a.logged_in() {
        (State::Needs, format!("log in once: {}", a.name()))
    } else if !miss.is_empty() {
        (State::Partial, format!("links ok{mcp}"))
    } else {
        (State::Ok, format!("{0}/{0} links · MCP in sync", LINKS.len()))
    };
    (Row { state, name: a.name(), detail }, plan)
}

fn detect(c: &Ctx) -> Report {
    let accts = c.accounts();
    if accts.is_empty() {
        return Report::new(State::Ok, "account 1 only · add more: --add-accounts N");
    }
    let mut plan = vec![];
    let mut rows = vec![];
    for a in &accts {
        let (r, p) = row(c, a);
        rows.push(r);
        plan.extend(p);
    }
    let wired = launchers_wired(c);
    if !wired {
        plan.push(format!("append to {}: {}", c.bashrc.display(), launcher_line(c)));
    }
    let names: Vec<String> = accts.iter().map(Account::name).collect();
    let state = if rows.iter().any(|r| r.state == State::Broken) {
        State::Broken
    } else if !wired || rows.iter().any(|r| r.state == State::Partial) {
        State::Partial
    } else {
        State::Ok
    };
    let launch = if wired { "launchers in ~/.bashrc" } else { "launchers missing from ~/.bashrc" };
    Report { state, detail: format!("{} · {launch}", names.join(" ")), rows, plan }
}

fn next_numbers(c: &Ctx, n: u32) -> Vec<u32> {
    let top = c.accounts().last().map_or(1, |a| a.n);
    (top + 1..=top + n).collect()
}

/// Preview lines for `--add-accounts n`.
pub fn add_plan(c: &Ctx, n: u32) -> Vec<String> {
    let mut v: Vec<String> = next_numbers(c, n)
        .into_iter()
        .map(|k| {
            format!(
                "create {}/.claude-account{k} (0700) + {} links into {}; then by hand: claude{k} (log in)",
                c.home.display(),
                LINKS.len(),
                c.claude_dir.display()
            )
        })
        .collect();
    if !launchers_wired(c) && c.accounts().is_empty() {
        v.push(format!("append to {}: {}", c.bashrc.display(), launcher_line(c)));
    }
    v
}

fn apply(c: &Ctx, o: &Opts) -> Result<(), String> {
    for k in next_numbers(c, o.add_accounts) {
        let d = c.home.join(format!(".claude-account{k}"));
        fs::DirBuilder::new().mode(0o700).create(&d).map_err(|e| format!("{}: {e}", d.display()))?;
        println!("  created {}", d.display());
    }
    let accts = c.accounts();
    let mut err = None;
    for a in &accts {
        for f in links(c, a).absent {
            symlink(c.claude_dir.join(f), a.dir.join(f)).map_err(|e| format!("{}: {e}", a.dir.join(f).display()))?;
        }
        if !a.logged_in() {
            println!("  log in once as {}, then re-run to copy MCP servers", a.name());
        } else if !missing_mcp(c, a).is_empty() {
            let base = servers(&c.claude_json);
            if let Err(e) = json::edit(&a.dir.join(".claude.json"), |v| {
                let mut merged = base;
                for (k, s) in v["mcpServers"].as_object().cloned().unwrap_or_default() {
                    merged.insert(k, s);
                }
                v["mcpServers"] = Value::Object(merged);
            }) {
                err = Some(e);
            }
        }
    }
    if !accts.is_empty() && !launchers_wired(c) {
        use std::io::Write;
        let mut f = fs::OpenOptions::new().append(true).create(true).open(&c.bashrc).map_err(|e| e.to_string())?;
        writeln!(f, "\n# Claude account launchers: claude, claude1, claudeN (claude-strapon)\n{}", launcher_line(c)).map_err(|e| e.to_string())?;
    }
    err.map_or(Ok(()), Err)
}
