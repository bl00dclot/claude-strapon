//! claude-strapon — measure the Claude Code stack on this machine, then fix what is not there.
//!
//!   scan -> board -> pick -> preview -> confirm -> apply -> rescan
//!
//! The rescan, not apply's result, is the proof.

mod comp;
mod ctx;
mod json;
mod ui;

use comp::{Component, Opts, Report, State};
use ctx::Ctx;
use std::collections::HashSet;

struct Args {
    dry: bool,
    yes: bool,
    only: Vec<String>,
    redo: Vec<String>,
    add: Option<u32>,
}

fn usage(comps: &[Component]) -> String {
    let names: Vec<&str> = comps.iter().map(|c| c.name).collect();
    format!(
        "usage: claude-strapon [--dry-run] [--yes] [--only a,b] [--redo a,b] [--add-accounts N]

  (no flags)        scan, show the board, pick fixes, preview, confirm, apply, rescan
  --dry-run         stop after the preview; change nothing
  --yes             take the pre-ticked fixes and apply without prompting
  --only a,b        limit everything to these components (ticks optional ones too)
  --redo a,b        offer these even when they already read ok
  --add-accounts N  create N more accounts (~/.claude-accountN, launcher claudeN)

Without a terminal and without --yes it stops after the preview.
Components: {}",
        names.join(" ")
    )
}

fn parse(argv: &[String]) -> Result<Option<Args>, String> {
    let mut a = Args { dry: false, yes: false, only: vec![], redo: vec![], add: None };
    let mut it = argv.iter();
    let list = |v: Option<&String>, flag: &str| -> Result<Vec<String>, String> {
        let v = v.ok_or(format!("{flag} needs a list"))?;
        Ok(v.split(',').filter(|s| !s.is_empty()).map(String::from).collect())
    };
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--dry-run" => a.dry = true,
            "--yes" | "-y" => a.yes = true,
            "--only" => a.only = list(it.next(), "--only")?,
            "--redo" => a.redo = list(it.next(), "--redo")?,
            "--add-accounts" => {
                let n = it.next().and_then(|v| v.parse().ok()).ok_or("--add-accounts needs a number")?;
                a.add = Some(n);
            }
            "-h" | "--help" => return Ok(None),
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(Some(a))
}

fn hostname() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname").map(|s| s.trim().to_string()).unwrap_or_default()
}

fn scan(ctx: &Ctx, comps: &[Component]) -> Vec<Report> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let r = std::thread::scope(|s| {
        let hs: Vec<_> = comps.iter().map(|c| s.spawn(move || (c.detect)(ctx))).collect();
        hs.into_iter().map(|h| h.join().unwrap_or_else(|_| Report::new(State::Broken, "detector failed"))).collect()
    });
    std::panic::set_hook(hook);
    r
}

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    let ctx = Ctx::from_env();
    let all = comp::all();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let a = match parse(&argv) {
        Ok(Some(a)) => a,
        Ok(None) => {
            println!("{}", usage(&all));
            return 0;
        }
        Err(e) => {
            eprintln!("{e}\n{}", usage(&all));
            return 2;
        }
    };
    for n in a.only.iter().chain(&a.redo) {
        if !all.iter().any(|c| c.name == n) {
            let hint = if n == "claude2" { " (now: accounts)" } else { "" };
            eprintln!("unknown component: {n}{hint}\n{}", usage(&all));
            return 2;
        }
    }
    let add = a.add.unwrap_or(0);
    let comps: Vec<Component> = all
        .into_iter()
        .filter(|c| a.only.is_empty() || a.only.iter().any(|n| n == c.name) || (add > 0 && c.name == "accounts"))
        .collect();

    let mut reports = scan(&ctx, &comps);
    ui::board(&format!("claude-strapon · {} · scan", hostname()), &comps, &reports);

    let interactive = ui::interactive();
    let (cand, pre) = comp::pick_defaults(&comps, &reports, &a.only, &a.redo, interactive, add);
    if cand.is_empty() {
        println!("\nNothing to fix.");
        return 0;
    }
    let picked: Vec<usize> = if !a.yes && interactive {
        let names: Vec<&str> = cand.iter().map(|&i| comps[i].name).collect();
        ui::choose_multi("Fix which? (space toggles, enter confirms)", &names, &pre).into_iter().map(|k| cand[k]).collect()
    } else {
        cand.iter().zip(&pre).filter(|(_, p)| **p).map(|(&i, _)| i).collect()
    };
    if picked.is_empty() {
        println!("\nNothing picked.");
        return 0;
    }
    let names: HashSet<&str> = picked.iter().map(|&i| comps[i].name).collect();
    let mut opts = Opts { add_accounts: add };
    if names.contains("accounts") && a.add.is_none() && interactive && !a.yes {
        opts.add_accounts = ui::ask_number("Add how many accounts?");
    }

    ui::head("preview");
    for &i in &picked {
        println!("\n  {}", comps[i].name);
        for l in &reports[i].plan {
            println!("    {l}");
        }
        if comps[i].name == "accounts" {
            for l in comp::accounts::add_plan(&ctx, opts.add_accounts) {
                println!("    {l}");
            }
        }
    }
    let sessions = ctx::out(std::process::Command::new("pgrep").args(["-x", "claude"])).map_or(0, |s| s.lines().count());
    if sessions > 0 && ["caveman", "codegraph", "accounts"].iter().any(|n| names.contains(n)) {
        println!(
            "\n  {}",
            ui::warn(&format!(
                "{sessions} claude session(s) running: they rewrite .claude.json and can undo MCP changes.\n  Close them before applying, or re-run afterwards to check."
            ))
        );
    }
    if ctx.root.join(".git").is_file() {
        println!(
            "\n  {}",
            ui::warn(&format!(
                "{} is a git worktree: hooks and ~/.bashrc lines will point into it. Apply from the main checkout.",
                ctx.root.display()
            ))
        );
    }
    println!("\n  Every JSON write is backed up first (*.bak-<timestamp>) and validated before it lands.");

    if a.dry {
        println!("\nDry run: nothing changed.");
        return 0;
    }
    if !a.yes {
        if !interactive {
            println!("\nNo terminal: stopping after the preview. Re-run with --yes to apply.");
            return 0;
        }
        if !ui::confirm("Apply these?") {
            println!("Nothing changed.");
            return 0;
        }
    }

    let mut failed: HashSet<&str> = HashSet::new();
    for &i in &picked {
        let c = &comps[i];
        ui::head(&format!("apply · {}", c.name));
        if let Some(d) = c.deps.iter().find(|d| failed.contains(*d)) {
            println!("  skipped: {d} failed above");
            failed.insert(c.name);
            continue;
        }
        if let Err(e) = (c.apply)(&ctx, &opts) {
            println!("  {} {e}", ui::bad("failed:"));
            failed.insert(c.name);
        }
    }

    reports = scan(&ctx, &comps);
    ui::board("after apply", &comps, &reports);
    if picked.iter().all(|&i| reports[i].state == State::Ok) {
        println!("\nAll picked components read ok.");
        0
    } else {
        println!("\nSome picked components still do not read ok (see above).");
        1
    }
}
