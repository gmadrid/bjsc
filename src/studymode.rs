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
            StudyMode::All => StudyMode::Hard,
            StudyMode::Hard => StudyMode::Soft,
            StudyMode::Soft => StudyMode::Splits,
            StudyMode::Splits => StudyMode::Doubles,
            StudyMode::Doubles => StudyMode::Drill,
            StudyMode::Drill => StudyMode::All,
        }
    }

    /// Whether this mode constructs specific hands rather than dealing from a shoe.
    pub fn is_constructed(&self) -> bool {
        !matches!(self, StudyMode::All)
    }
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
