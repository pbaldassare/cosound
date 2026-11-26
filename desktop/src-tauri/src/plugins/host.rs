//! Runs `cosound-auhost`, COSOUND's isolated Audio Unit host (source: `auhost/main.swift`, built by build.rs on macOS).
//! Every call is a separate process with a time limit: a plug-in that crashes or hangs is killed and reported,
//! COSOUND keeps running. macOS only; on other systems `available()` is false and callers say so.
use serde_json::Value;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Where the helper is: next to the app executable (bundled app, `tauri dev`), else where build.rs compiled it.
pub fn path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("COSOUND_AUHOST") {
        return Some(PathBuf::from(p)).filter(|p| p.exists());
    }
    let beside = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("cosound-auhost"))).filter(|p| p.exists());
    beside.or_else(|| option_env!("COSOUND_AUHOST_BUILT").map(PathBuf::from).filter(|p| p.exists()))
}

pub fn available() -> bool {
    cfg!(target_os = "macos") && path().is_some()
}

/// Run one helper command with a JSON request. Errors are plain text, ready to be shown and logged.
pub fn run(cmd: &str, req: &Value, timeout_s: u64) -> Result<Value, String> {
    let exe = path().ok_or("the plug-in host (cosound-auhost) is not available on this system")?;
    let mut child = Command::new(exe)
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not start the plug-in host: {e}"))?;
    child.stdin.take().map(|mut s| s.write_all(req.to_string().as_bytes()));
    let mut out = child.stdout.take().ok_or("no output pipe")?;
    let reader = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out.read_to_string(&mut s);
        s
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) if start.elapsed() > Duration::from_secs(timeout_s) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("the plug-in did not answer within {timeout_s} s and was stopped (hang); COSOUND is not affected"));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(e.to_string()),
        }
    };
    let text = reader.join().unwrap_or_default();
    let v: Value = match serde_json::from_str(text.trim()) {
        Ok(v) => v,
        Err(_) => {
            let mut err = String::new();
            child.stderr.take().map(|mut e| e.read_to_string(&mut err));
            return Err(match status.code() {
                None => "the plug-in crashed the host process (isolated: COSOUND is not affected)".to_string(),
                Some(c) => format!("the plug-in host failed (exit {c}){}", if err.is_empty() { String::new() } else { format!(": {}", err.trim()) }),
            });
        }
    };
    if v["ok"].as_bool() == Some(true) {
        Ok(v)
    } else {
        Err(v["error"].as_str().unwrap_or("unknown error").to_string())
    }
}
