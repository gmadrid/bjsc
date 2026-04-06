pub mod api;
mod bjerror;
pub mod card;
mod gamestate;
pub mod hand;
mod shoe;
mod strat;

mod hand_builder;
pub mod persistence;
pub mod progress;
mod studymode;
pub mod supabase;
mod table_index_keys;

pub use bjerror::*;
pub use gamestate::stats::Stats;
pub use gamestate::{AnswerResult, GameState};
pub use hand::Hand;
pub use hand_builder::build_hand_for_index;
pub use persistence::SavedState;
pub use spaced_rep::{DeckSummary, BOX_LABELS};
pub use strat::{
    all_charts, all_phrases, phrase_for_row, Action, ChartAction, StrategyChart, TableIndex,
    TableType,
};
pub use studymode::StudyMode;
pub use supabase::{AuthSession, SupabaseConfig};
pub use table_index_keys::{indices_for_mode, keys_for_mode};

/// Format a duration in seconds as a human-readable string (e.g. "2m 30s", "1h 5m").
pub fn format_wait_time(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        let m = secs / 60;
        let s = secs % 60;
        if s == 0 {
            format!("{}m", m)
        } else {
            format!("{}m {}s", m, s)
        }
    } else {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        if m == 0 {
            format!("{}h", h)
        } else {
            format!("{}h {}m", h, m)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_wait_time_seconds() {
        assert_eq!("0s", format_wait_time(0));
        assert_eq!("30s", format_wait_time(30));
        assert_eq!("59s", format_wait_time(59));
    }

    #[test]
    fn format_wait_time_minutes() {
        assert_eq!("1m", format_wait_time(60));
        assert_eq!("2m 30s", format_wait_time(150));
        assert_eq!("59m 59s", format_wait_time(3599));
    }

    #[test]
    fn format_wait_time_hours() {
        assert_eq!("1h", format_wait_time(3600));
        assert_eq!("1h 30m", format_wait_time(5400));
        assert_eq!("2h 5m", format_wait_time(7500));
    }
}
