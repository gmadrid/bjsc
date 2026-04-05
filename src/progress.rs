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

        // Trouble spots: sort by wrong count descending, take top 10
        let mut trouble: Vec<(String, u32, u32)> = per_index
            .into_iter()
            .filter(|(_, (wrong, _))| *wrong > 0)
            .map(|(idx, (wrong, seen))| (idx, wrong, seen))
            .collect();
        trouble.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
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
