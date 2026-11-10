//! The one safe way to rewrite a Claude JSON file.
//!
//! Claude Code silently discards a settings.json it cannot parse, which switches off every
//! hook in every account at once. So: resolve symlinks (account N links into ~/.claude),
//! refuse a file that does not parse now, back it up under a name no other backup in this
//! run can take, write a tmp file in the same dir, re-parse it, then rename over.

use serde_json::Value;
use std::fs;
use std::io::{ErrorKind, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn read(p: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(p).ok()?).ok()
}

pub fn edit(p: &Path, f: impl FnOnce(&mut Value)) -> Result<(), String> {
    let file = resolve(p);
    let shown = file.display();
    let (mut v, existed) = match fs::read_to_string(&file) {
        Ok(s) => (
            serde_json::from_str(&s).map_err(|e| format!("{shown} does not parse ({e}); not touching it"))?,
            true,
        ),
        Err(e) if e.kind() == ErrorKind::NotFound => (Value::Object(Default::default()), false),
        Err(e) => return Err(format!("{shown}: {e}")),
    };
    f(&mut v);
    let text = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())? + "\n";
    serde_json::from_str::<Value>(&text).map_err(|e| format!("refusing to write unparseable JSON to {shown}: {e}"))?;
    if existed {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        fs::copy(&file, backup_path(&file, now)).map_err(|e| format!("backup of {shown}: {e}"))?;
    }
    let tmp = sibling(&file, &format!(".tmp.{}", std::process::id()));
    let write = || -> std::io::Result<()> {
        let mut t = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&tmp)?;
        t.write_all(text.as_bytes())?;
        t.sync_all()?;
        fs::rename(&tmp, &file)
    };
    write().map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("writing {shown}: {e}")
    })
}

/// Follow symlinks to the real file, including a link whose target does not exist yet.
fn resolve(p: &Path) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_| match fs::read_link(p) {
        Ok(t) => p.parent().unwrap_or(Path::new("/")).join(t),
        Err(_) => p.to_path_buf(),
    })
}

fn sibling(p: &Path, suffix: &str) -> PathBuf {
    let mut s = p.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

/// `<file>.bak-YYYYmmdd-HHMMSS`, then `-2`, `-3`… if taken.
fn backup_path(p: &Path, secs: u64) -> PathBuf {
    let base = sibling(p, &format!(".bak-{}", stamp(secs)));
    if !base.exists() {
        return base;
    }
    (2..).map(|i| sibling(&base, &format!("-{i}"))).find(|c| !c.exists()).expect("unbounded")
}

/// UTC `YYYYmmdd-HHMMSS`.
fn stamp(secs: u64) -> String {
    // days -> civil date, Howard Hinnant's algorithm
    let (days, rem) = ((secs / 86400) as i64, secs % 86400);
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", rem / 3600, rem / 60 % 60, rem % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("strapon-json-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }
    fn baks(d: &Path) -> Vec<PathBuf> {
        let mut v: Vec<_> = fs::read_dir(d)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.to_string_lossy().contains(".bak-"))
            .collect();
        v.sort();
        v
    }

    #[test]
    fn stamp_formats_utc() {
        assert_eq!(stamp(0), "19700101-000000");
        assert_eq!(stamp(1762768330), "20251110-095210");
    }

    #[test]
    fn edit_preserves_key_order() {
        let d = tmpdir("order");
        let f = d.join("s.json");
        fs::write(&f, r#"{"z":1,"a":2}"#).unwrap();
        edit(&f, |v| v["m"] = "x".into()).unwrap();
        let v = read(&f).unwrap();
        let keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
        assert_eq!(keys, ["z", "a", "m"]);
    }

    #[test]
    fn edit_refuses_unparseable() {
        let d = tmpdir("bad");
        let f = d.join("s.json");
        fs::write(&f, "{bad").unwrap();
        assert!(edit(&f, |v| v["m"] = 1.into()).is_err());
        assert_eq!(fs::read_to_string(&f).unwrap(), "{bad");
        let left: Vec<_> = fs::read_dir(&d).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(left, ["s.json"], "tmp or backup left behind");
    }

    #[test]
    fn edit_creates_missing_file() {
        let d = tmpdir("new");
        let f = d.join("s.json");
        edit(&f, |v| v["m"] = 1.into()).unwrap();
        assert_eq!(read(&f).unwrap()["m"], 1);
    }

    #[test]
    fn edit_writes_through_symlink() {
        let d = tmpdir("link");
        let real = d.join("real.json");
        let link = d.join("link.json");
        fs::write(&real, "{}").unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();
        edit(&link, |v| v["m"] = 1.into()).unwrap();
        assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
        assert_eq!(read(&real).unwrap()["m"], 1);
    }

    #[test]
    fn edit_mode_0600() {
        let d = tmpdir("mode");
        let f = d.join("s.json");
        fs::write(&f, "{}").unwrap();
        fs::set_permissions(&f, fs::Permissions::from_mode(0o644)).unwrap();
        edit(&f, |v| v["m"] = 1.into()).unwrap();
        assert_eq!(fs::metadata(&f).unwrap().permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn backups_never_collide() {
        let d = tmpdir("bak");
        let f = d.join("s.json");
        fs::write(&f, r#"{"orig":true}"#).unwrap();
        edit(&f, |v| v["a"] = 1.into()).unwrap();
        edit(&f, |v| v["b"] = 1.into()).unwrap();
        let b = baks(&d);
        assert_eq!(b.len(), 2, "{b:?}");
        let first = b.iter().find(|p| !p.to_string_lossy().ends_with("-2")).unwrap();
        assert_eq!(fs::read_to_string(first).unwrap(), r#"{"orig":true}"#);
    }

    #[test]
    fn backup_path_suffixes() {
        let d = tmpdir("suffix");
        let f = d.join("s.json");
        let a = backup_path(&f, 0);
        assert!(a.to_string_lossy().ends_with("s.json.bak-19700101-000000"));
        fs::write(&a, "").unwrap();
        assert!(backup_path(&f, 0).to_string_lossy().ends_with(".bak-19700101-000000-2"));
    }
}
