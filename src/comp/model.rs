//! model: default model in settings.json. Per session, /model still overrides it.

use super::{Component, Report, State};
use crate::ctx::Ctx;
use crate::json;

pub const C: Component = Component { name: "model", deps: &["claude-code"], optional: false, always_offer: false, detect, apply: |c, _| apply(c) };

const MODEL: &str = "opus[1m]";

fn detect(c: &Ctx) -> Report {
    let r = match c.settings_json()["model"].as_str() {
        Some(MODEL) => Report::new(State::Ok, MODEL),
        Some(m) => Report::new(State::Partial, format!("{m} (wanted {MODEL})")),
        None => Report::new(State::Missing, "not pinned"),
    };
    r.plan([format!("settings.json: model = {MODEL}")])
}

fn apply(c: &Ctx) -> Result<(), String> {
    json::edit(&c.settings, |s| s["model"] = MODEL.into())
}
