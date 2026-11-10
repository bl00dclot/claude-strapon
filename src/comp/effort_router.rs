//! effort-router: UserPromptSubmit hook that tags each prompt with a complexity level.
//! The hook runs this repo's copy, by absolute path.

use super::{Component, Report, State};
use crate::ctx::Ctx;
use crate::json;
use serde_json::{Value, json};

pub const C: Component = Component { name: "effort-router", deps: &["claude-code"], optional: false, always_offer: false, detect, apply: |c, _| apply(c) };

fn script(c: &Ctx) -> String {
    c.root.join("files/effort-router.sh").display().to_string()
}

fn runs_router(entry: &Value) -> bool {
    entry["hooks"].as_array().into_iter().flatten().any(|h| h["command"].as_str().is_some_and(|s| s.contains("effort-router")))
}

fn detect(c: &Ctx) -> Report {
    let s = c.settings_json();
    let want = script(c);
    let cmds: Vec<String> = s["hooks"]["UserPromptSubmit"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|e| e["hooks"].as_array().into_iter().flatten())
        .filter_map(|h| h["command"].as_str().filter(|s| s.contains("effort-router")).map(String::from))
        .collect();
    let r = if cmds == [want.clone()] {
        Report::new(State::Ok, format!("hook -> {want}"))
    } else if !cmds.is_empty() {
        Report::new(State::Partial, format!("hook points elsewhere: {}", cmds.join(" ")))
    } else {
        Report::new(State::Missing, "no UserPromptSubmit hook")
    };
    r.plan([format!("settings.json: drop UserPromptSubmit entries running effort-router, add {want} (timeout 10)")])
}

fn apply(c: &Ctx) -> Result<(), String> {
    let cmd = script(c);
    json::edit(&c.settings, |s| {
        if !s["hooks"].is_object() {
            s["hooks"] = json!({});
        }
        let mut list: Vec<Value> = s["hooks"]["UserPromptSubmit"].as_array().cloned().unwrap_or_default();
        list.retain(|e| !runs_router(e));
        list.push(json!({"hooks": [{"type": "command", "command": cmd, "timeout": 10}]}));
        s["hooks"]["UserPromptSubmit"] = Value::Array(list);
    })
}
