use crate::studymode::StudyMode;
use serde::{Deserialize, Serialize};
use spaced_rep::Deck;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SavedState {
    #[serde(default)]
    pub mode: StudyMode,
    #[serde(default)]
    pub deck: Deck,
}

fn state_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".bjsc").join("state.toml")
}

pub fn load_state() -> SavedState {
    let path = state_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    } else {
        SavedState::default()
    }
}

pub fn save_state(state: &SavedState) {
    let path = state_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(content) = toml::to_string_pretty(state) {
        let _ = fs::write(&path, content);
    }
}
