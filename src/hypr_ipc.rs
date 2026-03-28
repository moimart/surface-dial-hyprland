use std::process::Command;

pub fn hypr_dispatch(dispatcher: &str, args: &str) -> Result<(), String> {
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
