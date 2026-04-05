use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StudyMode {
    #[default]
    All,
    Hard,
    Soft,
    Splits,
    Doubles,
    Drill,
}

impl StudyMode {
    pub fn next(self) -> Self {
        match self {
            StudyMode::All => StudyMode::Drill,
            StudyMode::Drill => StudyMode::Hard,
            StudyMode::Hard => StudyMode::Soft,
            StudyMode::Soft => StudyMode::Splits,
            StudyMode::Splits => StudyMode::Doubles,
            StudyMode::Doubles => StudyMode::All,
        }
    }

    /// Whether this mode constructs specific hands rather than dealing from a shoe.
    pub fn is_constructed(&self) -> bool {
        !matches!(self, StudyMode::All)
    }

    /// Stable short key for serialization/round-tripping (e.g., in HTML select elements).
    pub fn key(&self) -> &'static str {
        match self {
            StudyMode::All => "all",
            StudyMode::Hard => "hard",
            StudyMode::Soft => "soft",
            StudyMode::Splits => "splits",
            StudyMode::Doubles => "doubles",
            StudyMode::Drill => "drill",
        }
    }

    /// Parse from the stable short key.
    pub fn from_key(s: &str) -> Option<Self> {
        match s {
            "all" => Some(StudyMode::All),
            "hard" => Some(StudyMode::Hard),
            "soft" => Some(StudyMode::Soft),
            "splits" => Some(StudyMode::Splits),
            "doubles" => Some(StudyMode::Doubles),
            "drill" => Some(StudyMode::Drill),
            _ => None,
        }
    }

    /// All variants in display order.
    pub const ALL: [StudyMode; 6] = [
        StudyMode::All,
        StudyMode::Drill,
        StudyMode::Hard,
        StudyMode::Soft,
        StudyMode::Splits,
        StudyMode::Doubles,
    ];
}

impl Display for StudyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            StudyMode::All => "All (from shoe)",
            StudyMode::Hard => "Hard Totals",
            StudyMode::Soft => "Soft Totals",
            StudyMode::Splits => "Splits",
            StudyMode::Doubles => "Doubles",
            StudyMode::Drill => "Drill (spaced rep)",
        };
        write!(f, "{}", s)
    }
}
