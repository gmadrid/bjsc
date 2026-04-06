use crate::supabase::AnswerLogEntry;
use std::collections::HashMap;

/// Computed progress stats from answer log entries.
#[derive(Debug, Clone, Default)]
pub struct ProgressStats {
    pub total_answers: u32,
    pub total_correct: u32,
    pub accuracy_pct: f64,

    pub hard_total: u32,
    pub hard_correct: u32,
    pub soft_total: u32,
    pub soft_correct: u32,
    pub split_total: u32,
    pub split_correct: u32,
    pub double_total: u32,
    pub double_correct: u32,

    /// Top trouble spots: (table_index, times_wrong, times_seen)
    pub trouble_spots: Vec<(String, u32, u32)>,

    /// Recent sessions: (date_string, total, correct)
    pub sessions: Vec<(String, u32, u32)>,
}

impl ProgressStats {
    pub fn from_logs(logs: &[AnswerLogEntry]) -> Self {
        if logs.is_empty() {
            return Self::default();
        }

        let mut total = 0u32;
        let mut correct = 0u32;

        let mut hard_total = 0u32;
        let mut hard_correct = 0u32;
        let mut soft_total = 0u32;
        let mut soft_correct = 0u32;
        let mut split_total = 0u32;
        let mut split_correct = 0u32;
        let mut double_total = 0u32;
        let mut double_correct = 0u32;

        // Per-index tracking: (wrong_count, total_count)
        let mut per_index: HashMap<String, (u32, u32)> = HashMap::new();

        // Per-day tracking: (total, correct)
        let mut per_day: HashMap<String, (u32, u32)> = HashMap::new();

        for log in logs {
            total += 1;
            if log.correct {
                correct += 1;
            }

            // Category breakdown
            let category = log.table_index.split(':').next().unwrap_or("");
            match category {
                "hard" => {
                    hard_total += 1;
                    if log.correct {
                        hard_correct += 1;
                    }
                }
                "soft" => {
                    soft_total += 1;
                    if log.correct {
                        soft_correct += 1;
                    }
                }
                "split" => {
                    split_total += 1;
                    if log.correct {
                        split_correct += 1;
                    }
                }
                _ => {}
            }

            // Double is cross-cutting
            if log.correct_action == "Double" || log.player_action == "Double" {
                double_total += 1;
                if log.correct {
                    double_correct += 1;
                }
            }

            // Per-index
            let entry = per_index.entry(log.table_index.clone()).or_default();
            entry.1 += 1;
            if !log.correct {
                entry.0 += 1;
            }

            // Per-day (extract date from created_at)
            let day = log.created_at.split('T').next().unwrap_or("unknown");
            let day_entry = per_day.entry(day.to_string()).or_default();
            day_entry.0 += 1;
            if log.correct {
                day_entry.1 += 1;
            }
        }

        // Trouble spots: sort by wrong percentage descending, take top 10
        let mut trouble: Vec<(String, u32, u32)> = per_index
            .into_iter()
            .filter(|(_, (wrong, _))| *wrong > 0)
            .map(|(idx, (wrong, seen))| (idx, wrong, seen))
            .collect();
        trouble.sort_by(|a, b| {
            let pct_a = a.1 as f64 / a.2 as f64;
            let pct_b = b.1 as f64 / b.2 as f64;
            pct_b
                .partial_cmp(&pct_a)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.1.cmp(&a.1))
        });
        trouble.truncate(10);

        // Sessions: sort by date descending, take last 14 days
        let mut sessions: Vec<(String, u32, u32)> = per_day
            .into_iter()
            .map(|(day, (t, c))| (day, t, c))
            .collect();
        sessions.sort_by(|a, b| b.0.cmp(&a.0));
        sessions.truncate(14);

        let accuracy_pct = if total > 0 {
            correct as f64 / total as f64 * 100.0
        } else {
            0.0
        };

        ProgressStats {
            total_answers: total,
            total_correct: correct,
            accuracy_pct,
            hard_total,
            hard_correct,
            soft_total,
            soft_correct,
            split_total,
            split_correct,
            double_total,
            double_correct,
            trouble_spots: trouble,
            sessions,
        }
    }

    pub fn category_pct(correct: u32, total: u32) -> String {
        if total == 0 {
            return "—".to_string();
        }
        format!(
            "{:.0}% ({}/{})",
            correct as f64 / total as f64 * 100.0,
            correct,
            total
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::supabase::AnswerLogEntry;

    fn make_entry(
        table_index: &str,
        correct: bool,
        player_action: &str,
        correct_action: &str,
        created_at: &str,
    ) -> AnswerLogEntry {
        AnswerLogEntry {
            table_index: table_index.to_string(),
            correct,
            player_action: player_action.to_string(),
            correct_action: correct_action.to_string(),
            created_at: created_at.to_string(),
        }
    }

    // --- category_pct ---

    #[test]
    fn category_pct_zero_total_returns_dash() {
        assert_eq!("—", ProgressStats::category_pct(0, 0));
    }

    #[test]
    fn category_pct_all_correct() {
        assert_eq!("100% (10/10)", ProgressStats::category_pct(10, 10));
    }

    #[test]
    fn category_pct_all_wrong() {
        assert_eq!("0% (0/5)", ProgressStats::category_pct(0, 5));
    }

    #[test]
    fn category_pct_partial() {
        // 3 correct out of 4 = 75%
        assert_eq!("75% (3/4)", ProgressStats::category_pct(3, 4));
    }

    // --- from_logs() with empty slice ---

    #[test]
    fn from_logs_empty_returns_defaults() {
        let stats = ProgressStats::from_logs(&[]);
        assert_eq!(0, stats.total_answers);
        assert_eq!(0, stats.total_correct);
        assert_eq!(0.0, stats.accuracy_pct);
        assert!(stats.trouble_spots.is_empty());
        assert!(stats.sessions.is_empty());
    }

    // --- from_logs(): accuracy ---

    #[test]
    fn from_logs_accuracy_all_correct() {
        let logs = vec![
            make_entry("hard:12,5", true, "Stand", "Stand", "2024-01-01T10:00:00Z"),
            make_entry("hard:13,6", true, "Stand", "Stand", "2024-01-01T10:01:00Z"),
        ];
        let stats = ProgressStats::from_logs(&logs);
        assert_eq!(2, stats.total_answers);
        assert_eq!(2, stats.total_correct);
        assert!((stats.accuracy_pct - 100.0).abs() < 0.01);
    }

    #[test]
    fn from_logs_accuracy_mixed() {
        let logs = vec![
            make_entry("hard:12,5", true, "Stand", "Stand", "2024-01-01T10:00:00Z"),
            make_entry("hard:13,6", false, "Hit", "Stand", "2024-01-01T10:01:00Z"),
            make_entry("hard:14,7", false, "Hit", "Stand", "2024-01-01T10:02:00Z"),
            make_entry("hard:15,8", true, "Stand", "Stand", "2024-01-01T10:03:00Z"),
        ];
        let stats = ProgressStats::from_logs(&logs);
        assert_eq!(4, stats.total_answers);
        assert_eq!(2, stats.total_correct);
        assert!((stats.accuracy_pct - 50.0).abs() < 0.01);
    }

    // --- from_logs(): category breakdown ---

    #[test]
    fn from_logs_hard_category_counted() {
        let logs = vec![
            make_entry("hard:12,5", true, "Stand", "Stand", "2024-01-01T10:00:00Z"),
            make_entry("hard:13,6", false, "Hit", "Stand", "2024-01-01T10:01:00Z"),
        ];
        let stats = ProgressStats::from_logs(&logs);
        assert_eq!(2, stats.hard_total);
        assert_eq!(1, stats.hard_correct);
        assert_eq!(0, stats.soft_total);
        assert_eq!(0, stats.split_total);
    }

    #[test]
    fn from_logs_soft_category_counted() {
        let logs = vec![
            make_entry(
                "soft:17,3",
                true,
                "Double",
                "Double",
                "2024-01-01T10:00:00Z",
            ),
            make_entry(
                "soft:18,6",
                false,
                "Stand",
                "Double",
                "2024-01-01T10:01:00Z",
            ),
        ];
        let stats = ProgressStats::from_logs(&logs);
        assert_eq!(2, stats.soft_total);
        assert_eq!(1, stats.soft_correct);
        assert_eq!(0, stats.hard_total);
        assert_eq!(0, stats.split_total);
    }

    #[test]
    fn from_logs_split_category_counted() {
        let logs = vec![make_entry(
            "split:8,5",
            true,
            "Split",
            "Split",
            "2024-01-01T10:00:00Z",
        )];
        let stats = ProgressStats::from_logs(&logs);
        assert_eq!(1, stats.split_total);
        assert_eq!(1, stats.split_correct);
        assert_eq!(0, stats.hard_total);
        assert_eq!(0, stats.soft_total);
    }

    #[test]
    fn from_logs_unknown_category_prefix_ignored_in_buckets() {
        // A table_index that doesn't start with hard/soft/split
        let logs = vec![make_entry(
            "surrender:16,9",
            false,
            "Hit",
            "Surrender",
            "2024-01-01T10:00:00Z",
        )];
        let stats = ProgressStats::from_logs(&logs);
        assert_eq!(1, stats.total_answers);
        assert_eq!(0, stats.hard_total);
        assert_eq!(0, stats.soft_total);
        assert_eq!(0, stats.split_total);
    }

    // --- from_logs(): double cross-cutting ---

    #[test]
    fn from_logs_double_counted_when_correct_action_is_double() {
        let logs = vec![make_entry(
            "hard:11,5",
            true,
            "Double",
            "Double",
            "2024-01-01T10:00:00Z",
        )];
        let stats = ProgressStats::from_logs(&logs);
        assert_eq!(1, stats.double_total);
        assert_eq!(1, stats.double_correct);
    }

    #[test]
    fn from_logs_double_counted_when_player_action_is_double() {
        // Player played Double but was wrong (correct was Hit); still counts as a double attempt
        let logs = vec![make_entry(
            "hard:9,8",
            false,
            "Double",
            "Hit",
            "2024-01-01T10:00:00Z",
        )];
        let stats = ProgressStats::from_logs(&logs);
        assert_eq!(1, stats.double_total);
        assert_eq!(0, stats.double_correct);
    }

    #[test]
    fn from_logs_double_not_counted_for_non_double_entries() {
        let logs = vec![make_entry(
            "hard:16,9",
            false,
            "Stand",
            "Hit",
            "2024-01-01T10:00:00Z",
        )];
        let stats = ProgressStats::from_logs(&logs);
        assert_eq!(0, stats.double_total);
        assert_eq!(0, stats.double_correct);
    }

    // --- from_logs(): trouble spots ordering ---

    #[test]
    fn from_logs_trouble_spots_sorted_by_error_rate_descending() {
        let logs = vec![
            // index A: 1 wrong out of 2 (50%)
            make_entry("hard:12,5", false, "Hit", "Stand", "2024-01-01T10:00:00Z"),
            make_entry("hard:12,5", true, "Stand", "Stand", "2024-01-01T10:01:00Z"),
            // index B: 2 wrong out of 2 (100%)
            make_entry("hard:13,6", false, "Hit", "Stand", "2024-01-01T10:02:00Z"),
            make_entry("hard:13,6", false, "Hit", "Stand", "2024-01-01T10:03:00Z"),
        ];
        let stats = ProgressStats::from_logs(&logs);
        // B (100%) should come before A (50%)
        assert!(!stats.trouble_spots.is_empty());
        let first_key = &stats.trouble_spots[0].0;
        assert_eq!("hard:13,6", first_key);
    }

    #[test]
    fn from_logs_trouble_spots_excludes_fully_correct_indices() {
        let logs = vec![
            make_entry("hard:12,5", true, "Stand", "Stand", "2024-01-01T10:00:00Z"),
            make_entry("hard:12,5", true, "Stand", "Stand", "2024-01-01T10:01:00Z"),
        ];
        let stats = ProgressStats::from_logs(&logs);
        assert!(stats.trouble_spots.is_empty());
    }

    #[test]
    fn from_logs_trouble_spots_capped_at_ten() {
        // Create 12 distinct indices each with one wrong answer
        let mut logs = Vec::new();
        for row in 8u8..=17 {
            for col in [2u8, 3] {
                let idx = format!("hard:{},{}", row, col);
                logs.push(make_entry(
                    &idx,
                    false,
                    "Hit",
                    "Stand",
                    "2024-01-01T10:00:00Z",
                ));
                logs.push(make_entry(
                    &idx,
                    false,
                    "Hit",
                    "Stand",
                    "2024-01-01T10:01:00Z",
                ));
            }
        }
        let stats = ProgressStats::from_logs(&logs);
        assert!(stats.trouble_spots.len() <= 10);
    }

    // --- from_logs(): session grouping by date ---

    #[test]
    fn from_logs_sessions_grouped_by_date() {
        let logs = vec![
            make_entry("hard:12,5", true, "Stand", "Stand", "2024-01-01T10:00:00Z"),
            make_entry("hard:13,6", false, "Hit", "Stand", "2024-01-01T11:00:00Z"),
            make_entry("hard:14,7", true, "Stand", "Stand", "2024-01-02T10:00:00Z"),
        ];
        let stats = ProgressStats::from_logs(&logs);
        assert_eq!(2, stats.sessions.len());
    }

    #[test]
    fn from_logs_sessions_sorted_descending_by_date() {
        let logs = vec![
            make_entry("hard:12,5", true, "Stand", "Stand", "2024-01-01T10:00:00Z"),
            make_entry("hard:14,7", true, "Stand", "Stand", "2024-01-03T10:00:00Z"),
            make_entry("hard:13,6", true, "Stand", "Stand", "2024-01-02T10:00:00Z"),
        ];
        let stats = ProgressStats::from_logs(&logs);
        assert_eq!(3, stats.sessions.len());
        // Most recent date should come first
        assert_eq!("2024-01-03", stats.sessions[0].0);
    }

    #[test]
    fn from_logs_session_totals_correct_for_a_day() {
        let logs = vec![
            make_entry("hard:12,5", true, "Stand", "Stand", "2024-01-01T10:00:00Z"),
            make_entry("hard:13,6", false, "Hit", "Stand", "2024-01-01T11:00:00Z"),
            make_entry("hard:14,7", true, "Stand", "Stand", "2024-01-01T12:00:00Z"),
        ];
        let stats = ProgressStats::from_logs(&logs);
        let session = &stats.sessions[0];
        assert_eq!("2024-01-01", session.0); // date
        assert_eq!(3, session.1); // total
        assert_eq!(2, session.2); // correct
    }

    #[test]
    fn from_logs_sessions_capped_at_fourteen() {
        let mut logs = Vec::new();
        for day in 1u8..=20 {
            let date = format!("2024-01-{:02}T10:00:00Z", day);
            logs.push(make_entry("hard:12,5", true, "Stand", "Stand", &date));
        }
        let stats = ProgressStats::from_logs(&logs);
        assert!(stats.sessions.len() <= 14);
    }
}
