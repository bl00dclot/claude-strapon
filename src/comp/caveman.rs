//! caveman: compression proxy on 127.0.0.1:8787 that Claude sessions route through, its
//! native hooks, and its recovery MCP server. Optional.
//!
//! The route is set per launch from the shell (files/claude-route.sh, sourced from
//! ~/.bashrc), not by settings.json env: Remote Control refuses a non-api.anthropic.com
//! base URL and settings env beats the shell, so a settings route blocks Remote Control
//! for every account. `caveman claude` writes the settings route on first enable; apply
//! moves it. A route with nothing listening on :8787 fails every routed session, so apply
//! never writes one while the port is down, and the proxy gets a systemd user unit.

use super::{Component, Report, State};
use crate::ctx::{self, Ctx};
use crate::json;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub const C: Component = Component { name: "caveman", deps: &["node22", "claude-code"], optional: true, always_offer: false, detect, apply: |c, _| apply(c) };

const VERSION: &str = "2.1.0";
const ROUTE: &str = "http://127.0.0.1:8787/w/claude";
const UNIT: &str = "caveman-proxy.service";

fn cli(c: &Ctx) -> Option<PathBuf> {
    ctx::which("caveman").or_else(|| c.node22_bin().map(|b| b.join("caveman")).filter(|p| ctx::is_exec(p)))
}
fn up() -> bool {
    ctx::out(Command::new("ss").args(["-tlnH", "sport = :8787"])).is_some_and(|s| !s.trim().is_empty())
}
fn bins(c: &Ctx) -> bool {
    ctx::is_exec(&c.home.join(".caveman/bin/caveman-proxy"))
}
fn settings_route(s: &Value) -> bool {
    s["env"]["ANTHROPIC_BASE_URL"] == ROUTE
}
fn rc_file(c: &Ctx) -> PathBuf {
    c.root.join("files/claude-route.sh")
}
fn rc_line(c: &Ctx) -> String {
    format!(". \"{}\"", rc_file(c).display())
}
fn shell_route(c: &Ctx) -> bool {
    let line = rc_line(c);
    fs::read_to_string(&c.bashrc).unwrap_or_default().lines().any(|l| l.trim() == line)
        && fs::read_to_string(rc_file(c)).unwrap_or_default().contains(ROUTE)
}
fn hooks(s: &Value) -> bool {
    let Some(events) = s["hooks"].as_object() else { return false };
    events
        .values()
        .filter_map(Value::as_array)
        .flatten()
        .filter_map(|e| e["hooks"].as_array())
        .flatten()
        .filter_map(|h| h["command"].as_str())
        .any(|cmd| cmd.find("caveman-proxy").is_some_and(|i| cmd[i..].contains("native-hook")))
}
fn unit() -> bool {
    ctx::ok(Command::new("systemctl").args(["--user", "is-enabled", UNIT]))
}
fn mcp() -> bool {
    ctx::ok(ctx::claude_cmd(None).args(["mcp", "get", "caveman"]))
}
fn unit_path(c: &Ctx) -> PathBuf {
    c.home.join(".config/systemd/user").join(UNIT)
}

fn plan(c: &Ctx, s: &Value, has_cli: bool, is_up: bool, has_unit: bool, has_mcp: bool) -> Vec<String> {
    let mut p = vec![];
    if !has_cli {
        p.push(format!("npm i -g @caveman-ai/cli@{VERSION}      (with Node 22 first on PATH)"));
    }
    if !has_unit {
        p.push(format!("install files/{UNIT} -> {}", unit_path(c).display()));
        p.push(format!("systemctl --user daemon-reload && systemctl --user enable --now {UNIT}"));
        if is_up {
            p.push("  (a proxy already holds :8787; the unit retries every 15 s and takes over if it exits)".into());
        }
    }
    if bins(c) {
        let (sh, st) = (shell_route(c), settings_route(s));
        if !sh {
            p.push(format!("append to {}: {}   (only while something listens on :8787)", c.bashrc.display(), rc_line(c)));
        }
        if st {
            p.push(format!(
                "remove env.ANTHROPIC_BASE_URL + env._CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL from {}   (Remote Control refuses a proxied base URL)",
                c.settings.display()
            ));
        }
        if !sh || st {
            p.push("  then open a new shell: running sessions keep the old route".into());
        }
    }
    if !has_mcp {
        p.push("caveman mcp install claude      (recovery MCP in account 1; accounts copies it to the others)".into());
    }
    if !(bins(c) && hooks(s)) {
        p.push("then by hand: caveman claude      (installs proxy binaries, hooks), re-run to move its route".into());
    }
    p
}

fn detect(c: &Ctx) -> Report {
    let s = c.settings_json();
    let has_cli = cli(c).is_some();
    let (is_up, has_unit, has_mcp) = std::thread::scope(|sc| {
        let m = sc.spawn(mcp);
        (up(), unit(), m.join().unwrap_or(false))
    });
    let p = plan(c, &s, has_cli, is_up, has_unit, has_mcp);
    if !has_cli {
        return Report::new(State::Missing, "caveman CLI not installed").plan(p);
    }
    if (settings_route(&s) || shell_route(c)) && !is_up {
        return Report::new(State::Broken, "route set but nothing on :8787 — every claude session fails").plan(p);
    }
    let (b, mut fix, mut you) = (bins(c), vec![], vec![]);
    if !b {
        you.push("proxy binaries");
    }
    if !hooks(&s) {
        you.push("hooks");
    }
    if b && !has_unit {
        fix.push("not a service".to_string());
    }
    if settings_route(&s) {
        fix.push("route in settings.json blocks Remote Control".into());
    }
    if b && !shell_route(c) {
        fix.push("shell route missing".into());
    }
    if !has_mcp {
        fix.push("MCP 'caveman' missing".into());
    }
    let port = if is_up { ":8787 up" } else { ":8787 down" };
    let r = if !fix.is_empty() {
        let by_hand = if you.is_empty() { String::new() } else { format!(" · run `caveman claude` once for: {}", you.join(" ")) };
        Report::new(State::Partial, format!("{port} · {}{by_hand}", fix.join(" · ")))
    } else if !you.is_empty() {
        Report::new(State::Needs, format!("{port} · run `caveman claude` once to install: {}", you.join(" ")))
    } else {
        Report::new(State::Ok, format!("{port} · shell route (Remote Control direct) · hooks · service · MCP"))
    };
    r.plan(p)
}

/// PATH with Node 22 first, so npm globals land in (and run from) it.
fn node_path(c: &Ctx) -> std::ffi::OsString {
    let mut v: Vec<PathBuf> = c.node22_bin().into_iter().collect();
    v.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
    std::env::join_paths(v).unwrap_or_default()
}

fn apply(c: &Ctx) -> Result<(), String> {
    if cli(c).is_none()
        && !ctx::run(Command::new("npm").args(["i", "-g", &format!("@caveman-ai/cli@{VERSION}")]).env("PATH", node_path(c)))
    {
        return Err("npm install of the caveman CLI failed".into());
    }
    let mut started = false;
    if bins(c) && !unit() {
        let dst = unit_path(c);
        fs::create_dir_all(dst.parent().unwrap_or(&c.home)).map_err(|e| e.to_string())?;
        fs::copy(c.root.join("files").join(UNIT), &dst).map_err(|e| format!("{}: {e}", dst.display()))?;
        ctx::run(Command::new("systemctl").args(["--user", "daemon-reload"]));
        // The bind fails while another proxy holds :8787; Restart= retries.
        ctx::run(Command::new("systemctl").args(["--user", "enable", "--now", UNIT]));
        started = true;
    }
    // Shell route first, then drop the settings one, so no step leaves claude unrouted.
    if bins(c) {
        if !shell_route(c) {
            let listening = up() || (started && (0..5).any(|_| {
                std::thread::sleep(std::time::Duration::from_secs(1));
                up()
            }));
            if !listening {
                return Err(format!(
                    "nothing listens on :8787, so the route was not written (a route with no proxy fails every session). \
                     Start it (systemctl --user start {UNIT}), then re-run."
                ));
            }
            use std::io::Write;
            let mut f = fs::OpenOptions::new().append(true).create(true).open(&c.bashrc).map_err(|e| e.to_string())?;
            writeln!(f, "\n# caveman route for claude/claudeN; Remote Control launches go direct (claude-strapon)\n{}", rc_line(c))
                .map_err(|e| e.to_string())?;
        }
        if settings_route(&c.settings_json()) {
            json::edit(&c.settings, |s| {
                if let Some(env) = s["env"].as_object_mut() {
                    env.remove("ANTHROPIC_BASE_URL");
                    env.remove("_CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL");
                }
            })?;
        }
    }
    if !mcp() {
        let bin = cli(c).ok_or("caveman CLI not found after install")?;
        if !ctx::run(Command::new(bin).args(["mcp", "install", "claude"]).env("PATH", node_path(c))) {
            return Err("caveman mcp install failed".into());
        }
    }
    if !(bins(c) && hooks(&c.settings_json())) {
        println!("  next, by hand: caveman claude   (installs proxy binaries, hooks), then re-run");
    }
    Ok(())
}
