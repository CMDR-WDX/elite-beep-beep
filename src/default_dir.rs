use anyhow::anyhow;
use std::path::PathBuf;

#[cfg(target_family = "unix")]
pub fn try_get_default_directory() -> anyhow::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(||  anyhow!("Failed to find home directory. As such, cannot try to infer Steam Prefix, LocalAppdata, and so on."))?;
    let steam_commander_history = home_dir.join(".steam/steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/AppData/Local/Frontier Developments/Elite Dangerous/CommanderHistory");
    if steam_commander_history.exists() {
        return Ok(steam_commander_history);
    }
    // TODO: Impl EGS, etc
    return Err(anyhow!("Failed to find the directory automatically. Please use the --directory_path to point to %LOCALAPPDATA%/Frontier Developments/Elite Dangerous/CommanderHistory"));
}

#[cfg(target_family = "windows")]
pub fn try_get_default_directory() -> anyhow::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        anyhow!(
            "Failed to find home directory. As such, cannot try to infer LocalAppdata, and so on.",
        )
    })?;
    eprintln!("Home dir is {:?}", home_dir);
    let commander_history =
        home_dir.join("AppData/Local/Frontier Developments/Elite Dangerous/CommanderHistory");
    eprint!("History Dir is {:?}", commander_history);
    eprint!(
        "Does History Dir exist? {}",
        match commander_history.exists() {
            true => "yes",
            false => "no",
        }
    );

    if commander_history.exists() {
        return Ok(commander_history);
    }

    return Err(anyhow!("Failed to find the directory automatically. Please use the --directory_path to point to %LOCALAPPDATA%/Frontier Developments/Elite Dangerous/CommanderHistory"));
}
