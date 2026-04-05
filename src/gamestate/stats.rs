use crate::strat::{Action, TableIndex, TableType};

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub question_count: u32,
    pub questions_wrong: u32,

    pub hard_count: u32,
    pub hard_wrong: u32,

    pub soft_count: u32,
    pub soft_wrong: u32,

    pub split_count: u32,
    pub splits_wrong: u32,

    pub double_count: u32,
    pub doubles_wrong: u32,
}

impl Stats {
    pub fn count(&mut self, wrong: bool, action: Action, table_index: &TableIndex) {
        self.question_count += 1;
        if wrong {
            self.questions_wrong += 1;
        }

        match table_index.table_type() {
            TableType::Hard | TableType::Surrender => {
                self.hard_count += 1;
                if wrong {
                    self.hard_wrong += 1;
                }
            }
            TableType::Soft => {
                self.soft_count += 1;
                if wrong {
                    self.soft_wrong += 1;
                }
            }
            TableType::Split => {
                self.split_count += 1;
                if wrong {
                    self.splits_wrong += 1;
                }
            }
        }

        // Doubles are cross-cutting (can come from hard or soft)
        if action == Action::Double {
            self.double_count += 1;
            if wrong {
                self.doubles_wrong += 1;
            }
        }
    }

    pub fn numbers_string(count: u32, wrong: u32) -> String {
        if count == 0 {
            return "—".to_string();
        }
        let pct = (count - wrong) as f64 / count as f64 * 100.0;
        format!("{}/{} ({:.0}%)", count - wrong, count, pct)
    }
}
