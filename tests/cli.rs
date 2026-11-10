//! Runs the built binary against a throwaway HOME. PATH is the stub dir alone (plus links
//! to the few real tools the stubs use), so a missing stub is "command not found", never
//! the real systemctl or claude. stdin is closed. Nothing here can reach the real ~/.claude.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

pub struct Sb {
    pub t: PathBuf,
    pub home: PathBuf,
    pub bin: PathBuf,
    pub root: PathBuf,
    start: SystemTime,
}

impl Drop for Sb {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.t);
    }
}

pub fn jq(file: &Path, filter: &str) -> String {
    let o = Command::new("jq").args(["-r", filter]).arg(file).output().unwrap();
    String::from_utf8_lossy(&o.stdout).trim_end().to_string()
}

impl Sb {
    pub fn new(tag: &str) -> Sb {
        let t = std::env::temp_dir().join(format!("strapon-cli-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&t);
        let home = t.join("home");
        let bin = t.join("bin");
        for d in [home.join(".claude"), home.join(".claude-account2"), bin.clone(), home.join(".nvm/versions/node/v22.9.0/bin")] {
            fs::create_dir_all(d).unwrap();
        }
        let sb = Sb { t, home, bin, root: PathBuf::from(ROOT), start: SystemTime::now() };
        for tool in ["bash", "jq", "cat", "sh", "grep"] {
            let real = ["/usr/bin", "/bin"].iter().map(|d| Path::new(d).join(tool)).find(|p| p.exists()).unwrap();
            symlink(real, sb.bin.join(tool)).unwrap();
        }
        sb.write_exec(&sb.home.join(".nvm/versions/node/v22.9.0/bin/node"), "echo v22.9.0");
        for c in ["npm", "curl", "sudo"] {
            sb.stub(c, &format!("echo \"stub {c} called: $*\" >&2; exit 99"));
        }
        sb.stub("ss", "exit 0"); // nothing listening on :8787
        sb.stub("systemctl", "exit 1");
        sb.stub("pgrep", "exit 1");
        sb.stub("rtk", r#"case $1 in gain) exit 0;; --version) echo "rtk 0.42.4";; init) printf "[ok] Hook: rtk hook claude\n[ok] settings.json: RTK hook configured\n";; esac"#);
        // claude: MCP servers live in the account's .claude.json; plugins in a json file.
        // Every mcp add/remove is logged with the account it ran as.
        sb.stub("claude", r#"
cfg=${CLAUDE_CONFIG_DIR:+$CLAUDE_CONFIG_DIR/.claude.json}; cfg=${cfg:-$HOME/.claude.json}
case "$1 $2" in
  "--version "*) echo "2.1.308 (Claude Code)";;
  "mcp get") jq -e --arg n "$3" '.mcpServers[$n]' "$cfg" >/dev/null 2>&1 || exit 1; echo "Status: ✔ Connected"; jq -r --arg n "$3" '.mcpServers[$n].args // empty | "Args: " + join(" ")' "$cfg";;
  "mcp add"|"mcp remove") echo "${CLAUDE_CONFIG_DIR:-acct1} $*" >> "$HOME/claude-mcp.log";;
  "plugin list") cat "$HOME/plugins.json";;
  "plugin marketplace") printf "  ❯ claude-plugins-official\n  ❯ caveman\n  ❯ ponytail\n";;
  "plugin "*) echo "$*" >> "$HOME/claude-plugin.log";;
  *) echo "stub claude: $*" >&2; exit 99;;
esac"#);
        fs::write(sb.home.join(".claude.json"), r#"{"mcpServers": {"codegraph": {}, "dune": {}}}"#).unwrap();
        fs::write(sb.home.join(".claude-account2/.claude.json"), r#"{"mcpServers": {"codegraph": {}}, "oauthAccount": {"x": 1}}"#).unwrap();
        sb.link_account(2);
        // A stale effort-router hook from the ops days, next to an unrelated hook that must survive.
        fs::write(
            sb.settings(),
            r#"{"hooks": {"UserPromptSubmit": [
              {"hooks": [{"type": "command", "command": "/old/ops/effort-router.sh"}]},
              {"hooks": [{"type": "command", "command": "other-hook"}]}]}}"#,
        )
        .unwrap();
        let mut plugins: Vec<String> = fs::read_to_string(format!("{ROOT}/files/plugins.txt"))
            .unwrap()
            .lines()
            .map(|l| l.split('#').next().unwrap().trim().to_string())
            .filter(|l| !l.is_empty())
            .map(|id| format!(r#"{{"id": "{id}", "scope": "user", "enabled": true}}"#))
            .collect();
        plugins.push(r#"{"id": "stripe@claude-plugins-official", "scope": "user", "enabled": true}"#.into());
        fs::write(sb.home.join("plugins.json"), format!("[{}]", plugins.join(","))).unwrap();
        let mut sb = sb;
        std::thread::sleep(std::time::Duration::from_millis(30)); // clear the coarse mtime tick
        sb.start = SystemTime::now();
        sb
    }

    pub fn settings(&self) -> PathBuf {
        self.home.join(".claude/settings.json")
    }

    pub fn link_account(&self, n: u32) {
        let d = self.home.join(format!(".claude-account{n}"));
        fs::create_dir_all(&d).unwrap();
        for f in ["settings.json", "settings.local.json", "CLAUDE.md", "RTK.md", "agents", "skills", "plugins"] {
            symlink(self.home.join(".claude").join(f), d.join(f)).unwrap();
        }
    }

    pub fn write_exec(&self, p: &Path, body: &str) {
        fs::write(p, format!("#!/bin/bash\n{body}\n")).unwrap();
        fs::set_permissions(p, fs::Permissions::from_mode(0o755)).unwrap();
    }

    pub fn stub(&self, name: &str, body: &str) {
        self.write_exec(&self.bin.join(name), body);
    }

    /// (exit code, stdout + stderr)
    pub fn run(&self, args: &[&str]) -> (i32, String) {
        let o = Command::new(env!("CARGO_BIN_EXE_claude-strapon"))
            .args(args)
            .env_clear()
            .env("HOME", &self.home)
            .env("PATH", &self.bin)
            .env("NVM_DIR", self.home.join(".nvm"))
            .env("STRAPON_ROOT", &self.root)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let out = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
        (o.status.code().unwrap_or(-1), out)
    }

    /// Files under HOME modified since the sandbox was built (logs excluded).
    pub fn changed(&self) -> Vec<String> {
        fn walk(d: &Path, since: SystemTime, out: &mut Vec<String>) {
            for e in fs::read_dir(d).into_iter().flatten().flatten() {
                let p = e.path();
                let m = fs::symlink_metadata(&p).unwrap();
                if m.is_dir() {
                    walk(&p, since, out);
                } else if m.modified().unwrap() > since && !p.to_string_lossy().ends_with(".log") {
                    out.push(p.display().to_string());
                }
            }
        }
        let mut v = vec![];
        walk(&self.home, self.start, &mut v);
        v
    }
}

fn has(out: &str, want: &str) {
    assert!(out.contains(want), "missing {want:?} in:\n{out}");
}

#[test]
fn dry_run_changes_nothing() {
    let sb = Sb::new("dry");
    let (code, out) = sb.run(&["--dry-run"]);
    assert_eq!(code, 0, "{out}");
    has(&out, "Dry run: nothing changed");
    has(&out, "hook points elsewhere");
    has(&out, &format!("install files/agents/state-author.md -> {}/.claude/agents/state-author.md", sb.home.display()));
    let (code, out) = sb.run(&[]);
    assert_eq!(code, 0, "{out}");
    has(&out, "No terminal: stopping after the preview");
    assert_eq!(sb.changed(), Vec::<String>::new());
}

#[test]
fn apply_file_components() {
    let sb = Sb::new("apply");
    let (code, out) = sb.run(&["--yes", "--only", "effort-router,model,agents-skills"]);
    assert_eq!(code, 0, "{out}");
    let s = sb.settings();
    assert_eq!(jq(&s, ".model"), "opus[1m]");
    assert_eq!(jq(&s, "[.hooks.UserPromptSubmit[].hooks[].command] | join(\",\")"), format!("other-hook,{ROOT}/files/effort-router.sh"));
    assert_eq!(
        fs::read(format!("{ROOT}/files/agents/state-author.md")).unwrap(),
        fs::read(sb.home.join(".claude/agents/state-author.md")).unwrap()
    );
    assert!(fs::read_dir(sb.home.join(".claude")).unwrap().flatten().any(|e| e.file_name().to_string_lossy().starts_with("settings.json.bak-")));
    has(&out, "All picked components read ok");
}

#[test]
fn rerun_is_noop() {
    let sb = Sb::new("noop");
    let only = ["--yes", "--only", "effort-router,model,agents-skills"];
    sb.run(&only);
    let baks = || fs::read_dir(sb.home.join(".claude")).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().contains(".bak-")).count();
    let n = baks();
    let (code, out) = sb.run(&only);
    assert_eq!(code, 0, "{out}");
    has(&out, "Nothing to fix");
    assert_eq!(baks(), n);
}

#[test]
fn statusline_follows_updates() {
    let sb = Sb::new("status");
    let p = sb.home.join(".claude/plugins/cache/caveman/caveman/abc123/src/hooks");
    fs::create_dir_all(&p).unwrap();
    fs::write(p.join("caveman-statusline.sh"), "").unwrap();
    let (code, out) = sb.run(&["--yes", "--only", "statusline"]);
    assert_eq!(code, 0, "{out}");
    has(&out, "caveman badge (follows plugin updates)");
}

#[test]
fn unknown_component_exits_2() {
    let sb = Sb::new("unknown");
    let (code, out) = sb.run(&["--only", "claude2"]);
    assert_eq!(code, 2, "{out}");
    has(&out, "unknown component: claude2 (now: accounts)");
}

#[test]
fn missing_tool_is_not_fatal() {
    let sb = Sb::new("notool");
    fs::remove_file(sb.bin.join("claude")).unwrap();
    let (code, out) = sb.run(&["--dry-run"]);
    assert_eq!(code, 0, "{out}");
    has(&out, "claude-code");
    has(&out, "not installed");
    has(&out, "hook points elsewhere");
}

#[test]
fn worktree_warning() {
    let mut sb = Sb::new("wt");
    let fake = sb.t.join("root");
    fs::create_dir_all(&fake).unwrap();
    symlink(format!("{ROOT}/files"), fake.join("files")).unwrap();
    sb.root = fake.clone();
    fs::write(fake.join(".git"), "gitdir: /elsewhere\n").unwrap();
    let (_, out) = sb.run(&["--dry-run", "--only", "model"]);
    has(&out, "is a git worktree");
    fs::remove_file(fake.join(".git")).unwrap();
    fs::create_dir(fake.join(".git")).unwrap();
    let (_, out) = sb.run(&["--dry-run", "--only", "model"]);
    assert!(!out.contains("is a git worktree"), "{out}");
}

// --- accounts -------------------------------------------------------------------------

fn mode(p: &Path) -> u32 {
    fs::metadata(p).unwrap().permissions().mode() & 0o777
}
fn launcher_line() -> String {
    format!(". \"{ROOT}/files/claude-accounts.sh\"")
}
fn bashrc_count(sb: &Sb, line: &str) -> usize {
    fs::read_to_string(sb.home.join(".bashrc")).unwrap_or_default().lines().filter(|l| l.trim() == line).count()
}

#[test]
fn accounts_merge_mcp() {
    let sb = Sb::new("acct-merge");
    let (code, out) = sb.run(&["--dry-run", "--only", "accounts"]);
    assert_eq!(code, 0, "{out}");
    has(&out, "MCP missing: dune");
    let (code, out) = sb.run(&["--yes", "--only", "accounts"]);
    assert_eq!(code, 0, "{out}");
    let a2 = sb.home.join(".claude-account2");
    assert!(fs::symlink_metadata(a2.join("settings.json")).unwrap().file_type().is_symlink());
    assert_eq!(jq(&a2.join(".claude.json"), ".mcpServers | keys | join(\",\")"), "codegraph,dune");
    assert_eq!(jq(&a2.join(".claude.json"), ".oauthAccount.x"), "1");
    assert_eq!(mode(&a2.join(".claude.json")), 0o600);
    assert_eq!(bashrc_count(&sb, &launcher_line()), 1);
    has(&out, "All picked components read ok");
}

#[test]
fn three_accounts() {
    let sb = Sb::new("acct-three");
    let a3 = sb.home.join(".claude-account3");
    fs::create_dir_all(&a3).unwrap();
    fs::write(a3.join(".claude.json"), r#"{"mcpServers": {}}"#).unwrap();
    let (_, out) = sb.run(&["--dry-run", "--only", "accounts"]);
    has(&out, "claude3");
    has(&out, "0/7 links");
    has(&out, "MCP missing: codegraph dune");
    let (code, out) = sb.run(&["--yes", "--only", "accounts"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(fs::read_link(a3.join("skills")).unwrap(), sb.home.join(".claude/skills"));
    assert_eq!(jq(&a3.join(".claude.json"), ".mcpServers | keys | join(\",\")"), "codegraph,dune");
}

#[test]
fn not_logged_in_is_needs() {
    let sb = Sb::new("acct-login");
    sb.link_account(4);
    let (code, out) = sb.run(&["--yes", "--only", "accounts"]);
    assert_eq!(code, 0, "{out}");
    has(&out, "log in once: claude4");
}

#[test]
fn add_accounts() {
    let sb = Sb::new("acct-add");
    sb.link_account(3);
    fs::write(sb.home.join(".claude-account3/.claude.json"), r#"{"mcpServers": {"codegraph": {}, "dune": {}}}"#).unwrap();
    let (code, out) = sb.run(&["--yes", "--only", "accounts", "--add-accounts", "2"]);
    assert_eq!(code, 0, "{out}");
    for n in [4, 5] {
        let d = sb.home.join(format!(".claude-account{n}"));
        assert_eq!(mode(&d), 0o700, "{n}");
        let links = fs::read_dir(&d).unwrap().flatten().filter(|e| e.file_type().unwrap().is_symlink()).count();
        assert_eq!(links, 7, "{n}");
        has(&out, &format!("log in once: claude{n}"));
    }
    assert!(!sb.home.join(".claude-account6").exists());
    assert_eq!(bashrc_count(&sb, &launcher_line()), 1);
    let (code, out) = sb.run(&["--yes", "--only", "accounts"]);
    assert_eq!(code, 0, "{out}");
    has(&out, "Nothing to fix");
    assert_eq!(bashrc_count(&sb, &launcher_line()), 1);
}

#[test]
fn add_accounts_dry_run() {
    let sb = Sb::new("acct-adddry");
    let (code, out) = sb.run(&["--dry-run", "--add-accounts", "1"]);
    assert_eq!(code, 0, "{out}");
    has(&out, &format!("create {}/.claude-account3", sb.home.display()));
    assert_eq!(sb.changed(), Vec::<String>::new());
}

#[test]
fn junk_dirs_ignored() {
    let sb = Sb::new("acct-junk");
    for j in [".claude-account02", ".claude-accountx", ".claude-account1", ".claude-account"] {
        fs::create_dir_all(sb.home.join(j)).unwrap();
    }
    let (code, out) = sb.run(&["--yes", "--only", "accounts"]);
    assert_eq!(code, 0, "{out}");
    has(&out, "claude2");
    for name in ["claude02", "claudex", "claude1 "] {
        assert!(!out.contains(name), "{name} in:\n{out}");
    }
    for j in [".claude-account02", ".claude-accountx", ".claude-account1", ".claude-account"] {
        assert_eq!(fs::read_dir(sb.home.join(j)).unwrap().count(), 0, "{j} touched");
    }
}

#[test]
fn real_file_blocks_link() {
    let sb = Sb::new("acct-real");
    let a3 = sb.home.join(".claude-account3");
    fs::create_dir_all(&a3).unwrap();
    fs::write(a3.join("settings.json"), "{\"mine\":1}").unwrap();
    fs::write(a3.join(".claude.json"), "{}").unwrap();
    let (code, out) = sb.run(&["--yes", "--only", "accounts"]);
    assert_eq!(code, 1, "{out}");
    has(&out, "not links, move aside by hand: settings.json");
    assert!(fs::symlink_metadata(a3.join("settings.json")).unwrap().is_file());
    assert_eq!(fs::read_to_string(a3.join("settings.json")).unwrap(), "{\"mine\":1}");
}

// --- rtk, caveman, codegraph, plugins --------------------------------------------------

const ROUTE: &str = "http://127.0.0.1:8787/w/claude";

fn route_line() -> String {
    format!(". \"{ROOT}/files/claude-route.sh\"")
}

fn set_json(p: &Path, filter: &str) {
    let o = Command::new("jq").arg(filter).arg(p).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    fs::write(p, o.stdout).unwrap();
}

/// caveman installed, proxy binaries + hooks + MCP present, settings route still in place.
fn caveman_ready(sb: &Sb) {
    sb.stub("caveman", "exit 0");
    sb.stub("systemctl", "exit 0");
    let b = sb.home.join(".caveman/bin");
    fs::create_dir_all(&b).unwrap();
    sb.write_exec(&b.join("caveman-proxy"), "");
    set_json(
        &sb.settings(),
        &format!(
            r#".env = {{ANTHROPIC_BASE_URL: "{ROUTE}", _CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL: "1", ENABLE_TOOL_SEARCH: "auto"}}
              | .hooks.SessionStart = [{{hooks: [{{type: "command", command: "caveman-proxy native-hook session_start"}}]}}]"#
        ),
    );
    set_json(&sb.home.join(".claude.json"), ".mcpServers.caveman = {}");
}

#[test]
fn optional_unticked() {
    let sb = Sb::new("optional");
    fs::remove_file(sb.bin.join("rtk")).unwrap();
    let (code, out) = sb.run(&["--dry-run"]);
    assert_eq!(code, 0, "{out}");
    for absent in ["npm i -g @caveman-ai/cli", "npm i -g @colbymchenry/codegraph", "rtk init -g"] {
        assert!(!out.contains(absent), "{absent} pre-ticked:\n{out}");
    }
    has(&sb.run(&["--dry-run", "--only", "caveman"]).1, "npm i -g @caveman-ai/cli@2.1.0");
    has(&sb.run(&["--dry-run", "--only", "codegraph"]).1, "npm i -g @colbymchenry/codegraph");
    has(&sb.run(&["--dry-run", "--only", "rtk"]).1, "rtk init -g --auto-patch");
}

#[test]
fn dead_route_broken() {
    let sb = Sb::new("deadroute");
    sb.stub("caveman", "exit 0");
    set_json(&sb.settings(), &format!(r#".env.ANTHROPIC_BASE_URL = "{ROUTE}""#));
    let (code, out) = sb.run(&["--dry-run", "--only", "caveman"]);
    assert_eq!(code, 0, "{out}");
    has(&out, "route set but nothing on :8787");
}

#[test]
fn route_moves_to_shell() {
    let sb = Sb::new("route");
    caveman_ready(&sb);
    sb.stub("ss", r#"echo "LISTEN 0 4096 127.0.0.1:8787 0.0.0.0:*""#);
    let (_, out) = sb.run(&["--dry-run", "--only", "caveman"]);
    has(&out, "route in settings.json blocks Remote Control");
    has(&out, &format!("append to {}/.bashrc: {}", sb.home.display(), route_line()));
    let (code, out) = sb.run(&["--yes", "--only", "caveman"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(jq(&sb.settings(), ".env | tojson"), r#"{"ENABLE_TOOL_SEARCH":"auto"}"#);
    assert_eq!(bashrc_count(&sb, &route_line()), 1);
    has(&out, "All picked components read ok");
    let (code, out) = sb.run(&["--yes", "--only", "caveman"]);
    assert_eq!(code, 0, "{out}");
    has(&out, "Nothing to fix");
    assert_eq!(bashrc_count(&sb, &route_line()), 1);
}

#[test]
fn route_not_written_while_down() {
    let sb = Sb::new("routedown");
    caveman_ready(&sb);
    set_json(&sb.settings(), "del(.env.ANTHROPIC_BASE_URL)");
    let (code, out) = sb.run(&["--yes", "--only", "caveman"]);
    assert_eq!(code, 1, "{out}");
    has(&out, "nothing listens on :8787");
    assert_eq!(bashrc_count(&sb, &route_line()), 0);
}

#[test]
fn plugins_diff() {
    let sb = Sb::new("plugins");
    let (_, out) = sb.run(&["--dry-run", "--only", "plugins"]);
    has(&out, "remove: stripe");
    has(&out, "claude plugin uninstall -s user -y stripe@claude-plugins-official");
    sb.run(&["--yes", "--only", "plugins"]);
    let log = fs::read_to_string(sb.home.join("claude-plugin.log")).unwrap_or_default();
    assert_eq!(log.trim(), "plugin uninstall -s user -y stripe@claude-plugins-official");
}

#[test]
fn codegraph_every_logged_in_account() {
    let sb = Sb::new("codegraph");
    let pkg = sb.home.join(".nvm/versions/node/v22.9.0/lib/node_modules/@colbymchenry/codegraph");
    fs::create_dir_all(&pkg).unwrap();
    fs::write(pkg.join("npm-shim.js"), "").unwrap();
    fs::write(pkg.join("package.json"), r#"{"version": "1.2.3"}"#).unwrap();
    sb.link_account(3);
    fs::write(sb.home.join(".claude-account3/.claude.json"), "{}").unwrap();
    sb.link_account(4); // not logged in
    let (_, out) = sb.run(&["--dry-run", "--only", "codegraph"]);
    has(&out, "1.2.3");
    has(&out, "claude3: not registered");
    sb.run(&["--yes", "--only", "codegraph"]);
    let log = fs::read_to_string(sb.home.join("claude-mcp.log")).unwrap();
    let adds: Vec<&str> = log.lines().filter(|l| l.contains("mcp add codegraph")).map(|l| l.split(' ').next().unwrap()).collect();
    let h = sb.home.display();
    assert_eq!(adds, ["acct1".to_string(), format!("{h}/.claude-account2"), format!("{h}/.claude-account3")]);
    assert!(log.contains("serve --mcp"), "{log}");
}
