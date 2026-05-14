use std::process::Command;
use std::sync::OnceLock;

/// Cached result of the Hyprland version probe. `true` once we've confirmed
/// the running Hyprland uses the Lua config parser (>= 0.55), `false` for
/// the legacy hyprlang parser (<= 0.54).
static USE_LUA: OnceLock<bool> = OnceLock::new();

fn detect_lua_parser() -> bool {
    // `hyprctl version` first line: "Hyprland 0.55.1 built from ..."
    let Some(out) = Command::new("hyprctl").arg("version").output().ok() else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let Some(line) = stdout.lines().next() else { return false };
    let Some(ver) = line.split_whitespace().nth(1) else { return false };
    let mut parts = ver.split('.');
    let major: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    major > 0 || minor >= 55
}

/// True when the running Hyprland uses the Lua config parser (>= 0.55).
/// Cached after the first call.
pub fn use_lua() -> bool {
    *USE_LUA.get_or_init(detect_lua_parser)
}

/// Run a Lua expression via `hyprctl eval`. Returns Err if Hyprland printed
/// an "error: ..." reply (note: `hyprctl eval` always exits 0).
pub fn hypr_eval(lua: &str) -> Result<(), String> {
    let output = Command::new("hyprctl")
        .arg("eval")
        .arg(lua)
        .output()
        .map_err(|e| format!("Failed to run hyprctl: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim_start();
    if trimmed.starts_with("error:") {
        return Err(format!("hyprctl eval: {}", trimmed.trim_end()));
    }
    Ok(())
}

/// Issue a Hyprland dispatcher call. Picks the right transport for the
/// running Hyprland version: legacy `hyprctl dispatch <name> <args>` on
/// 0.54 and earlier, `hyprctl eval 'hl.dispatch(hl.dsp.<name>(<args>))'`
/// on 0.55 and later.
///
/// Only the dispatchers this daemon actually uses are mapped to Lua
/// equivalents — add cases here as new callers appear.
pub fn hypr_dispatch(dispatcher: &str, args: &str) -> Result<(), String> {
    if use_lua() {
        let lua = match dispatcher {
            // hl.dsp.layout takes the message string as-is.
            "layoutmsg" => format!("hl.dispatch(hl.dsp.layout({}))", lua_string(args)),
            other => {
                return Err(format!(
                    "no Lua mapping for legacy dispatcher '{other}'; add a case in hypr_ipc::hypr_dispatch"
                ));
            }
        };
        return hypr_eval(&lua);
    }

    let output = Command::new("hyprctl")
        .arg("dispatch")
        .arg(dispatcher)
        .arg(args)
        .output()
        .map_err(|e| format!("Failed to run hyprctl: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("hyprctl failed: {stderr}"));
    }
    Ok(())
}

/// Quote a string for Lua source (handles backslash and double-quote).
fn lua_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}
