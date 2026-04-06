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

    /// Icon/emoji for each mode.
    pub fn icon(&self) -> &'static str {
        match self {
            StudyMode::All => "\u{1F0CF}",  // 🃏 joker
            StudyMode::Drill => "\u{25CE}", // ◎ bullseye
            StudyMode::Hard => "\u{1F4AA}", // 💪 flexed biceps
            StudyMode::Soft => "A2",
            StudyMode::Splits => "AA",
            StudyMode::Doubles => "\u{23EC}", // ⏬ double down
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- ALL constant ordering ---

    #[test]
    fn all_contains_all_six_variants() {
        assert_eq!(6, StudyMode::ALL.len());
    }

    #[test]
    fn all_starts_with_all_mode() {
        assert_eq!(StudyMode::All, StudyMode::ALL[0]);
    }

    #[test]
    fn all_ordering_matches_expected() {
        let expected = [
            StudyMode::All,
            StudyMode::Drill,
            StudyMode::Hard,
            StudyMode::Soft,
            StudyMode::Splits,
            StudyMode::Doubles,
        ];
        assert_eq!(expected, StudyMode::ALL);
    }

    #[test]
    fn all_contains_every_variant() {
        let all = StudyMode::ALL;
        assert!(all.contains(&StudyMode::All));
        assert!(all.contains(&StudyMode::Drill));
        assert!(all.contains(&StudyMode::Hard));
        assert!(all.contains(&StudyMode::Soft));
        assert!(all.contains(&StudyMode::Splits));
        assert!(all.contains(&StudyMode::Doubles));
    }

    // --- next() cycling ---

    #[test]
    fn next_cycles_through_all_variants_and_wraps() {
        let mut mode = StudyMode::All;
        // Collect the full cycle starting from All
        let mut visited = vec![mode];
        loop {
            mode = mode.next();
            if mode == StudyMode::All {
                break;
            }
            visited.push(mode);
        }
        // Should have visited all 6 variants exactly once before wrapping
        assert_eq!(6, visited.len());
    }

    #[test]
    fn next_all_returns_drill() {
        assert_eq!(StudyMode::Drill, StudyMode::All.next());
    }

    #[test]
    fn next_drill_returns_hard() {
        assert_eq!(StudyMode::Hard, StudyMode::Drill.next());
    }

    #[test]
    fn next_hard_returns_soft() {
        assert_eq!(StudyMode::Soft, StudyMode::Hard.next());
    }

    #[test]
    fn next_soft_returns_splits() {
        assert_eq!(StudyMode::Splits, StudyMode::Soft.next());
    }

    #[test]
    fn next_splits_returns_doubles() {
        assert_eq!(StudyMode::Doubles, StudyMode::Splits.next());
    }

    #[test]
    fn next_doubles_wraps_back_to_all() {
        assert_eq!(StudyMode::All, StudyMode::Doubles.next());
    }

    // --- key() / from_key() round-trip ---

    #[test]
    fn key_from_key_round_trip_for_all_variants() {
        for mode in StudyMode::ALL {
            let key = mode.key();
            let restored = StudyMode::from_key(key);
            assert_eq!(Some(mode), restored, "round-trip failed for key '{}'", key);
        }
    }

    #[test]
    fn key_values_are_correct() {
        assert_eq!("all", StudyMode::All.key());
        assert_eq!("hard", StudyMode::Hard.key());
        assert_eq!("soft", StudyMode::Soft.key());
        assert_eq!("splits", StudyMode::Splits.key());
        assert_eq!("doubles", StudyMode::Doubles.key());
        assert_eq!("drill", StudyMode::Drill.key());
    }

    #[test]
    fn from_key_returns_none_for_unknown_key() {
        assert_eq!(None, StudyMode::from_key("unknown"));
        assert_eq!(None, StudyMode::from_key(""));
        assert_eq!(None, StudyMode::from_key("ALL"));
        assert_eq!(None, StudyMode::from_key("Hard"));
    }

    #[test]
    fn from_key_all() {
        assert_eq!(Some(StudyMode::All), StudyMode::from_key("all"));
    }

    #[test]
    fn from_key_drill() {
        assert_eq!(Some(StudyMode::Drill), StudyMode::from_key("drill"));
    }

    // --- is_constructed() ---

    #[test]
    fn all_mode_is_not_constructed() {
        assert!(!StudyMode::All.is_constructed());
    }

    #[test]
    fn drill_mode_is_constructed() {
        assert!(StudyMode::Drill.is_constructed());
    }

    #[test]
    fn hard_mode_is_constructed() {
        assert!(StudyMode::Hard.is_constructed());
    }

    #[test]
    fn soft_mode_is_constructed() {
        assert!(StudyMode::Soft.is_constructed());
    }

    #[test]
    fn splits_mode_is_constructed() {
        assert!(StudyMode::Splits.is_constructed());
    }

    #[test]
    fn doubles_mode_is_constructed() {
        assert!(StudyMode::Doubles.is_constructed());
    }

    // --- default ---

    #[test]
    fn default_is_all() {
        assert_eq!(StudyMode::All, StudyMode::default());
    }
}
