//! Where each Claude account keeps its state, and how to run the tools that read it.
//!
//! Account 1 is the default config dir (~/.claude + ~/.claude.json). Account N ≥ 2 is
//! CLAUDE_CONFIG_DIR=~/.claude-accountN, whose settings/agents/skills/plugins are symlinks
//! into ~/.claude; only its .claude.json (login + MCP servers) is its own.

use serde_json::Value;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct Ctx {
    pub root: PathBuf,
    pub home: PathBuf,
    pub claude_dir: PathBuf,
    pub claude_json: PathBuf,
    pub settings: PathBuf,
    pub nvm_dir: PathBuf,
    pub bashrc: PathBuf,
}

impl Ctx {
    pub fn from_env() -> Ctx {
        let home = PathBuf::from(env::var_os("HOME").unwrap_or_else(|| "/".into()));
        let root = env::var_os("STRAPON_ROOT").map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")), PathBuf::from);
        let claude_dir = env::var_os("CLAUDE_DIR").map_or_else(|| home.join(".claude"), PathBuf::from);
        let nvm_dir = env::var_os("NVM_DIR").map(PathBuf::from).unwrap_or_else(|| {
            let c = home.join(".config/nvm");
            if c.is_dir() { c } else { home.join(".nvm") }
        });
        Ctx {
            claude_json: home.join(".claude.json"),
            settings: claude_dir.join("settings.json"),
            bashrc: home.join(".bashrc"),
            root,
            home,
            claude_dir,
            nvm_dir,
        }
    }

    /// settings.json, or `{}` when missing or unparseable.
    pub fn settings_json(&self) -> Value {
        crate::json::read(&self.settings).unwrap_or_else(|| Value::Object(Default::default()))
    }

    /// Newest Node 22 under nvm, as its bin dir.
    pub fn node22_bin(&self) -> Option<PathBuf> {
        let ver = |name: &str| -> Option<Vec<u64>> {
            name.strip_prefix("v22.")?.split('.').map(|p| p.parse().ok()).collect()
        };
        std::fs::read_dir(self.nvm_dir.join("versions/node"))
            .ok()?
            .filter_map(|e| {
                let name = e.ok()?.file_name().into_string().ok()?;
                Some((ver(&name)?, name))
            })
            .max()
            .map(|(_, name)| self.nvm_dir.join("versions/node").join(name).join("bin"))
    }

    /// Extra accounts (N ≥ 2), sorted by N.
    pub fn accounts(&self) -> Vec<Account> {
        let mut v: Vec<Account> = std::fs::read_dir(&self.home)
            .into_iter()
            .flatten()
            .filter_map(|e| {
                let e = e.ok()?;
                let n = account_number(e.file_name().to_str()?)?;
                e.path().is_dir().then(|| Account { n, dir: e.path() })
            })
            .collect();
        v.sort_by_key(|a| a.n);
        v
    }
}

/// `.claude-accountN` with N ≥ 2, digits only, no leading zero.
pub fn account_number(name: &str) -> Option<u32> {
    let d = name.strip_prefix(".claude-account")?;
    if d.is_empty() || d.starts_with('0') || !d.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    d.parse().ok().filter(|&n| n >= 2)
}

pub struct Account {
    pub n: u32,
    pub dir: PathBuf,
}

impl Account {
    pub fn name(&self) -> String {
        format!("claude{}", self.n)
    }
    pub fn logged_in(&self) -> bool {
        self.dir.join(".claude.json").is_file()
    }
}

/// `claude` as the account that owns `dir` (None = account 1).
pub fn claude_cmd(dir: Option<&Path>) -> Command {
    let mut c = Command::new("claude");
    match dir {
        None => c.env_remove("CLAUDE_CONFIG_DIR"),
        Some(d) => c.env("CLAUDE_CONFIG_DIR", d),
    };
    c
}

/// stdout when the command exits 0; quiet otherwise.
pub fn out(cmd: &mut Command) -> Option<String> {
    let o = cmd.stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).into_owned())
}

/// Exit 0, nothing printed.
pub fn ok(cmd: &mut Command) -> bool {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// For apply: output goes to the user; stdin stays theirs (installers may prompt).
pub fn run(cmd: &mut Command) -> bool {
    cmd.status().is_ok_and(|s| s.success())
}

pub fn which(name: &str) -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?)
        .map(|d| d.join(name))
        .find(|p| is_exec(p))
}

pub fn is_exec(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata().is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_numbers() {
        assert_eq!(account_number(".claude-account2"), Some(2));
        assert_eq!(account_number(".claude-account17"), Some(17));
        for junk in [".claude-account", ".claude-account1", ".claude-account0", ".claude-account02", ".claude-accountx", ".claude-account3x", "claude-account3"] {
            assert_eq!(account_number(junk), None, "{junk}");
        }
    }
}
