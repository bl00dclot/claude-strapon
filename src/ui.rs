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

/// Ticked indices into `items`: a ratatui list on a terminal, numbered prompts if it cannot start.
pub fn choose_multi(header: &str, items: &[&str], pre: &[bool]) -> Vec<usize> {
    tui_choose(header, items, pre).unwrap_or_else(|_| plain_choose(header, items, pre))
}

pub struct Picker {
    pub cursor: usize,
    pub on: Vec<bool>,
}

/// One key. `Some(ticked)` ends the pick: Enter confirms, Esc/q cancel (nothing ticked).
pub fn step(p: &mut Picker, key: ratatui::crossterm::event::KeyCode) -> Option<Vec<usize>> {
    use ratatui::crossterm::event::KeyCode::*;
    match key {
        Up | Char('k') => p.cursor = p.cursor.saturating_sub(1),
        Down | Char('j') => p.cursor = (p.cursor + 1).min(p.on.len().saturating_sub(1)),
        Char(' ') => {
            if let Some(o) = p.on.get_mut(p.cursor) {
                *o = !*o;
            }
        }
        Char('a') => {
            let all = p.on.iter().all(|&o| o);
            p.on.iter_mut().for_each(|o| *o = !all);
        }
        Enter => return Some((0..p.on.len()).filter(|&i| p.on[i]).collect()),
        Esc | Char('q') => return Some(vec![]),
        _ => {}
    }
    None
}

fn tui_choose(header: &str, items: &[&str], pre: &[bool]) -> std::io::Result<Vec<usize>> {
    use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Paragraph;
    use ratatui::{TerminalOptions, Viewport};

    let height = u16::try_from(items.len() + 3).unwrap_or(u16::MAX);
    let mut term = ratatui::try_init_with_options(TerminalOptions { viewport: Viewport::Inline(height) })?;
    let mut p = Picker { cursor: 0, on: pre.to_vec() };
    let mut pick = || -> std::io::Result<Vec<usize>> {
        loop {
            term.draw(|f| {
                let mut lines = vec![Line::styled(header, Style::new().fg(Color::Magenta))];
                for (i, it) in items.iter().enumerate() {
                    let here = i == p.cursor;
                    let style = if here { Style::new().add_modifier(Modifier::BOLD) } else { Style::new() };
                    lines.push(Line::from(vec![
                        Span::raw(if here { " › " } else { "   " }),
                        Span::styled(if p.on[i] { "[x] " } else { "[ ] " }, Style::new().fg(Color::Green)),
                        Span::styled(*it, style),
                    ]));
                }
                lines.push(Line::styled("↑/↓ move · space toggle · a all · enter confirm · esc cancel", Style::new().add_modifier(Modifier::DIM)));
                f.render_widget(Paragraph::new(lines), f.area());
            })?;
            if let Event::Key(k) = event::read()? {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
                    return Ok(vec![]);
                }
                if let Some(r) = step(&mut p, k.code) {
                    return Ok(r);
                }
            }
        }
    };
    let r = pick();
    ratatui::restore();
    r
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyCode;

    fn picker(on: &[bool]) -> Picker {
        Picker { cursor: 0, on: on.to_vec() }
    }

    #[test]
    fn moves_with_clamping() {
        let mut p = picker(&[false, false, false]);
        assert_eq!(step(&mut p, KeyCode::Up), None);
        assert_eq!(p.cursor, 0);
        step(&mut p, KeyCode::Down);
        step(&mut p, KeyCode::Char('j'));
        step(&mut p, KeyCode::Down);
        assert_eq!(p.cursor, 2);
        step(&mut p, KeyCode::Char('k'));
        assert_eq!(p.cursor, 1);
    }

    #[test]
    fn space_toggles_and_enter_returns_ticked() {
        let mut p = picker(&[true, false, false]);
        step(&mut p, KeyCode::Down);
        step(&mut p, KeyCode::Char(' '));
        assert_eq!(step(&mut p, KeyCode::Enter), Some(vec![0, 1]));
    }

    #[test]
    fn a_toggles_all() {
        let mut p = picker(&[true, false]);
        step(&mut p, KeyCode::Char('a'));
        assert_eq!(p.on, [true, true]);
        step(&mut p, KeyCode::Char('a'));
        assert_eq!(p.on, [false, false]);
    }

    #[test]
    fn esc_and_q_cancel() {
        assert_eq!(step(&mut picker(&[true]), KeyCode::Esc), Some(vec![]));
        assert_eq!(step(&mut picker(&[true]), KeyCode::Char('q')), Some(vec![]));
    }
}
