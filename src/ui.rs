//! Colour, the board, and prompts. Callers check `interactive()` before prompting at all,
//! so nothing here blocks on a closed stdin.

use crate::comp::{Component, Report, State};
use std::io::{BufRead, IsTerminal, Write};

pub fn interactive() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

fn paint(code: &str, s: &str) -> String {
    if std::io::stdout().is_terminal() { format!("\x1b[{code}m{s}\x1b[0m") } else { s.to_string() }
}

pub fn mark(s: State) -> String {
    match s {
        State::Ok => paint("32", "●"),
        State::Partial => paint("33", "◐"),
        State::Missing => paint("2", "○"),
        State::Broken => paint("31", "✗"),
        State::Needs => paint("33", "!"),
    }
}

pub fn warn(s: &str) -> String {
    paint("33", s)
}

pub fn bad(s: &str) -> String {
    paint("31", s)
}

pub fn head(s: &str) {
    println!("\n{}", paint("35", s));
}

pub fn board(title: &str, comps: &[Component], reports: &[Report]) {
    head(title);
    let line = |c: &Component, r: &Report| {
        println!("  {} {:<15} {}", mark(r.state), c.name, r.detail);
        for row in &r.rows {
            println!("      {} {:<11} {}", mark(row.state), row.name, row.detail);
        }
    };
    for (c, r) in comps.iter().zip(reports).filter(|(_, r)| r.state != State::Needs) {
        line(c, r);
    }
    let mut needs = comps.iter().zip(reports).filter(|(_, r)| r.state == State::Needs).peekable();
    if needs.peek().is_some() {
        println!("\n  {} (the wizard cannot do these)", warn("needs you"));
        needs.for_each(|(c, r)| line(c, r));
    }
}

fn read_line() -> String {
    let mut s = String::new();
    let _ = std::io::stdin().lock().read_line(&mut s);
    s.trim().to_string()
}

fn prompt(q: &str) -> String {
    eprint!("{q}");
    let _ = std::io::stderr().flush();
    read_line()
}

pub fn confirm(q: &str) -> bool {
    prompt(&format!("{q} [y/N] ")).to_lowercase().starts_with('y')
}

pub fn ask_number(q: &str) -> u32 {
    prompt(&format!("{q} [0] ")).parse().unwrap_or(0)
}

/// Ticked indices into `items`.
pub fn choose_multi(header: &str, items: &[&str], pre: &[bool]) -> Vec<usize> {
    plain_choose(header, items, pre)
}

fn plain_choose(header: &str, items: &[&str], pre: &[bool]) -> Vec<usize> {
    let mut on = pre.to_vec();
    loop {
        eprintln!("\n{header}");
        for (i, it) in items.iter().enumerate() {
            eprintln!("  {:2} [{}] {it}", i + 1, if on[i] { 'x' } else { ' ' });
        }
        let line = prompt("numbers to toggle (e.g. \"1 3\"), enter to accept: ");
        if line.is_empty() {
            break;
        }
        for n in line.split_whitespace().filter_map(|w| w.parse::<usize>().ok()) {
            if (1..=items.len()).contains(&n) {
                on[n - 1] = !on[n - 1];
            }
        }
    }
    (0..items.len()).filter(|&i| on[i]).collect()
}
