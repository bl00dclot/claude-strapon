//! statusline: caveman's mode badge. Cosmetic, so optional. The script lives under the
//! plugin cache, whose path changes on every caveman plugin update, so the command looks
//! up the newest copy each time it runs instead of pinning one.

use super::{Component, Report, State};
use crate::ctx::Ctx;
use crate::json;
use serde_json::json;
use std::path::{Path, PathBuf};

pub const C: Component = Component { name: "statusline", deps: &["plugins"], optional: true, always_offer: false, detect, apply: |c, _| apply(c) };

const CMD: &str = r#"bash -c 's=$(ls -1t "$HOME"/.claude/plugins/cache/caveman/caveman/*/src/hooks/caveman-statusline.sh 2>/dev/null | awk "NR==1"); [ -n "$s" ] && exec bash "$s"'"#;

fn script(c: &Ctx) -> Option<PathBuf> {
    let cache = c.claude_dir.join("plugins/cache/caveman/caveman");
    std::fs::read_dir(cache)
        .ok()?
        .flatten()
        .map(|e| e.path().join("src/hooks/caveman-statusline.sh"))
        .filter(|p| p.is_file())
        .max_by_key(|p| p.metadata().and_then(|m| m.modified()).ok())
}

/// `bash "<abs path>"` or `bash <abs path>`: a command pinned to one plugin version.
fn pinned(cmd: &str) -> Option<&str> {
    let rest = cmd.strip_prefix("bash")?;
    let rest = rest.strip_prefix(' ')?.trim_start();
    let p = rest.strip_prefix('"').unwrap_or(rest);
    let p = p.strip_suffix('"').unwrap_or(p);
    (p.starts_with('/') && !p.contains('"')).then_some(p)
}

fn detect(c: &Ctx) -> Report {
    let s = c.settings_json();
    let r = match s["statusLine"]["command"].as_str().filter(|s| !s.is_empty()) {
        Some(cmd) => match pinned(cmd) {
            Some(p) if Path::new(p).is_file() => Report::new(
                State::Partial,
                format!("pinned to one plugin version: {}", Path::new(p).file_name().unwrap_or_default().to_string_lossy()),
            ),
            Some(p) => Report::new(State::Broken, format!("missing script: {p} (plugin updated?)")),
            None if cmd == CMD => Report::new(State::Ok, "caveman badge (follows plugin updates)"),
            None => Report::new(State::Ok, format!("custom: {cmd}")),
        },
        None if script(c).is_some() => Report::new(State::Missing, "not configured"),
        None => Report::new(State::Missing, "not configured; caveman plugin not installed yet"),
    };
    r.plan([format!("settings.json: statusLine = {CMD}")])
}

fn apply(c: &Ctx) -> Result<(), String> {
    script(c).ok_or("caveman plugin not installed")?;
    json::edit(&c.settings, |s| s["statusLine"] = json!({"type": "command", "command": CMD}))
}

#[cfg(test)]
mod tests {
    #[test]
    fn pinned_forms() {
        assert_eq!(super::pinned(r#"bash "/a/b.sh""#), Some("/a/b.sh"));
        assert_eq!(super::pinned("bash  /a/b.sh"), Some("/a/b.sh"));
        assert_eq!(super::pinned(super::CMD), None);
        assert_eq!(super::pinned("bashx /a"), None);
    }
}
