// The launchers are bash; their test is bash. This runs it under `cargo test`.
#[test]
fn shell_launchers() {
    let o = std::process::Command::new("bash")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/shell.sh"))
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(o.status.success() && out.contains("PASS"), "{out}{}", String::from_utf8_lossy(&o.stderr));
}
