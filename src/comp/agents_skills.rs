//! agents + skills: the subagent roster (files/agents/*.md) and user skills
//! (files/skills/<name>/) copied into ~/.claude. Files there that are not ours stay.

use super::{Component, Report, State};
use crate::ctx::Ctx;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub const C: Component = Component { name: "agents-skills", deps: &["claude-code"], optional: false, always_offer: false, detect, apply: |c, _| apply(c) };

/// Paths relative to files/, sorted.
fn ours(c: &Ctx) -> Vec<PathBuf> {
    fn walk(base: &Path, d: &Path, out: &mut Vec<PathBuf>) {
        for e in fs::read_dir(d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else if let Ok(rel) = p.strip_prefix(base) {
                out.push(rel.to_path_buf());
            }
        }
    }
    let base = c.root.join("files");
    let mut v = vec![];
    walk(&base, &base.join("agents"), &mut v);
    walk(&base, &base.join("skills"), &mut v);
    v.sort();
    v
}

fn stale(c: &Ctx) -> Vec<PathBuf> {
    ours(c)
        .into_iter()
        .filter(|rel| fs::read(c.root.join("files").join(rel)).ok() != fs::read(c.claude_dir.join(rel)).ok())
        .collect()
}

/// agents/x.md -> x; skills/y/SKILL.md -> y
fn short(rel: &Path) -> String {
    let s = rel.to_string_lossy();
    let s = s.strip_suffix("/SKILL.md").or_else(|| s.strip_suffix(".md")).unwrap_or(&s);
    s.rsplit('/').next().unwrap_or(s).to_string()
}

fn detect(c: &Ctx) -> Report {
    let total = ours(c).len();
    let st = stale(c);
    let r = if st.is_empty() {
        Report::new(State::Ok, format!("{total} files current"))
    } else if st.len() == total {
        Report::new(State::Missing, format!("none installed ({total} files)"))
    } else {
        let names: Vec<String> = st.iter().map(|p| short(p)).collect();
        Report::new(State::Partial, format!("{} of {total} missing or outdated: {}", st.len(), names.join(" ")))
    };
    let dir = c.claude_dir.display();
    r.plan(st.iter().map(|rel| format!("install files/{0} -> {dir}/{0}", rel.display())))
}

fn apply(c: &Ctx) -> Result<(), String> {
    for rel in stale(c) {
        let dst = c.claude_dir.join(&rel);
        let go = || -> std::io::Result<()> {
            fs::create_dir_all(dst.parent().unwrap_or(Path::new("/")))?;
            fs::copy(c.root.join("files").join(&rel), &dst)?;
            fs::set_permissions(&dst, fs::Permissions::from_mode(0o644))
        };
        go().map_err(|e| format!("{}: {e}", dst.display()))?;
    }
    Ok(())
}
