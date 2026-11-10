//! Components: one per piece of the stack. `detect` is read-only and runs in parallel with
//! the others; it returns the state, a one-line detail, sub-rows, and the plan lines for the
//! facts it just measured. `apply` runs serially in `all()` order and measures again.

use crate::ctx::Ctx;

pub mod accounts;
mod agents_skills;
mod caveman;
mod claude_code;
mod codegraph;
mod effort_router;
mod model;
mod node22;
mod plugins;
mod rtk;
mod statusline;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Ok,
    Partial,
    Missing,
    Broken,
    Needs,
}

pub struct Row {
    pub state: State,
    pub name: String,
    pub detail: String,
}

pub struct Report {
    pub state: State,
    pub detail: String,
    pub rows: Vec<Row>,
    pub plan: Vec<String>,
}

impl Report {
    pub fn new(state: State, detail: impl Into<String>) -> Report {
        Report { state, detail: detail.into(), rows: vec![], plan: vec![] }
    }
    pub fn plan(mut self, lines: impl IntoIterator<Item = String>) -> Report {
        self.plan.extend(lines);
        self
    }
}

pub struct Opts {
    pub add_accounts: u32,
}

pub struct Component {
    pub name: &'static str,
    pub deps: &'static [&'static str],
    pub optional: bool,
    pub always_offer: bool,
    pub detect: fn(&Ctx) -> Report,
    pub apply: fn(&Ctx, &Opts) -> Result<(), String>,
}

pub fn all() -> Vec<Component> {
    vec![
        node22::C,
        claude_code::C,
        rtk::C,
        caveman::C,
        codegraph::C,
        accounts::C,
        effort_router::C,
        plugins::C,
        statusline::C,
        agents_skills::C,
        model::C,
    ]
}

/// Which components to offer, and which start ticked.
/// Offered: anything not ok and not a human-only step, plus `--redo`, plus an
/// always-offer component on a terminal, plus `accounts` when accounts are being added.
/// Ticked: non-optional ones, or any when `--only` named the set; an always-offer
/// component that reads ok starts unticked; `--add-accounts` ticks `accounts`.
pub fn pick_defaults(
    comps: &[Component],
    reports: &[Report],
    only: &[String],
    redo: &[String],
    interactive: bool,
    add: u32,
) -> (Vec<usize>, Vec<bool>) {
    let mut cand = vec![];
    let mut pre = vec![];
    for (i, (c, r)) in comps.iter().zip(reports).enumerate() {
        let redo_it = redo.iter().any(|n| n == c.name);
        let adding = c.name == "accounts" && add > 0;
        let quiet = matches!(r.state, State::Ok | State::Needs);
        let offered = !quiet || redo_it || adding || (c.always_offer && interactive && r.state == State::Ok);
        if !offered {
            continue;
        }
        cand.push(i);
        pre.push(if adding {
            true
        } else if quiet && !redo_it {
            false
        } else {
            !c.optional || !only.is_empty()
        });
    }
    (cand, pre)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comp(name: &'static str, optional: bool, always_offer: bool) -> Component {
        Component { name, deps: &[], optional, always_offer, detect: |_| Report::new(State::Ok, ""), apply: |_, _| Ok(()) }
    }
    fn picks(states: &[State], comps: &[Component], only: &[&str], redo: &[&str], tty: bool, add: u32) -> Vec<(&'static str, bool)> {
        let reports: Vec<Report> = states.iter().map(|&s| Report::new(s, "")).collect();
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let (idx, pre) = pick_defaults(comps, &reports, &s(only), &s(redo), tty, add);
        idx.iter().zip(pre).map(|(&i, p)| (comps[i].name, p)).collect()
    }

    #[test]
    fn optional_starts_unticked_unless_only() {
        let c = [comp("rtk", true, false), comp("model", false, false)];
        let st = [State::Missing, State::Missing];
        assert_eq!(picks(&st, &c, &[], &[], false, 0), [("rtk", false), ("model", true)]);
        assert_eq!(picks(&st[..1], &c[..1], &["rtk"], &[], false, 0), [("rtk", true)]);
    }

    #[test]
    fn ok_offered_only_on_redo_and_needs_never() {
        let c = [comp("model", false, false), comp("node22", false, false)];
        assert_eq!(picks(&[State::Ok, State::Needs], &c, &[], &[], false, 0), []);
        assert_eq!(picks(&[State::Ok, State::Needs], &c, &[], &["model"], false, 0), [("model", true)]);
    }

    #[test]
    fn always_offer_on_terminal_only() {
        let c = [comp("accounts", false, true)];
        assert_eq!(picks(&[State::Ok], &c, &[], &[], true, 0), [("accounts", false)]);
        assert_eq!(picks(&[State::Ok], &c, &[], &[], false, 0), []);
        assert_eq!(picks(&[State::Partial], &c, &[], &[], true, 0), [("accounts", true)]);
    }

    #[test]
    fn add_accounts_offers_and_ticks() {
        let c = [comp("accounts", false, true)];
        assert_eq!(picks(&[State::Ok], &c, &[], &[], false, 2), [("accounts", true)]);
    }
}
