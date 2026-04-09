use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::error::MeicanError;
use crate::models::Session;

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("meican")
}

fn session_path() -> PathBuf {
    config_dir().join("session.json")
}

pub fn save_session(session: &Session) -> Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;

    let path = session_path();
    let json = serde_json::to_string_pretty(session)?;
    fs::write(&path, json)
        .with_context(|| format!("Failed to write session file: {}", path.display()))?;
    Ok(())
}

pub fn load_session() -> Result<Session> {
    let path = session_path();
    if !path.exists() {
        return Err(MeicanError::NotLoggedIn.into());
    }
    let data = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read session file: {}", path.display()))?;
    let session: Session = serde_json::from_str(&data)?;
    Ok(session)
}

pub fn delete_session() -> Result<()> {
    let path = session_path();
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}
