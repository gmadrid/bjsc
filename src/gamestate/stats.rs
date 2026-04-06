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
    pub split_wrong: u32,

    pub double_count: u32,
    pub double_wrong: u32,
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
                    self.split_wrong += 1;
                }
            }
        }

        // Doubles are cross-cutting (can come from hard or soft)
        if action == Action::Double {
            self.double_count += 1;
            if wrong {
                self.double_wrong += 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strat::{new_table_index, ColIndex, RowIndex, TableIndex, TableType};

    fn make_table_index(tt: TableType, row: u8, col: u8) -> TableIndex {
        let ri = RowIndex::new(tt, row).unwrap();
        let ci: ColIndex = col.to_string().parse().unwrap();
        new_table_index(ri, ci)
    }

    // --- numbers_string ---

    #[test]
    fn numbers_string_zero_count_returns_dash() {
        assert_eq!("—", Stats::numbers_string(0, 0));
    }

    #[test]
    fn numbers_string_all_correct() {
        // 10 seen, 0 wrong -> 10/10 (100%)
        assert_eq!("10/10 (100%)", Stats::numbers_string(10, 0));
    }

    #[test]
    fn numbers_string_all_wrong() {
        // 5 seen, 5 wrong -> 0/5 (0%)
        assert_eq!("0/5 (0%)", Stats::numbers_string(5, 5));
    }

    #[test]
    fn numbers_string_partial_correct() {
        // 4 seen, 1 wrong -> 3/4 (75%)
        assert_eq!("3/4 (75%)", Stats::numbers_string(4, 1));
    }

    #[test]
    fn numbers_string_rounds_percentage() {
        // 3 seen, 1 wrong -> 2/3 (67%)
        assert_eq!("2/3 (67%)", Stats::numbers_string(3, 1));
    }

    // --- count(): total question / wrong tracking ---

    #[test]
    fn count_increments_question_count_on_correct() {
        let mut stats = Stats::default();
        let ti = make_table_index(TableType::Hard, 12, 5);
        stats.count(false, Action::Stand, &ti);
        assert_eq!(1, stats.question_count);
        assert_eq!(0, stats.questions_wrong);
    }

    #[test]
    fn count_increments_questions_wrong_on_wrong() {
        let mut stats = Stats::default();
        let ti = make_table_index(TableType::Hard, 12, 5);
        stats.count(true, Action::Hit, &ti);
        assert_eq!(1, stats.question_count);
        assert_eq!(1, stats.questions_wrong);
    }

    // --- count(): Hard category ---

    #[test]
    fn count_hard_correct_increments_hard_count_only() {
        let mut stats = Stats::default();
        let ti = make_table_index(TableType::Hard, 14, 7);
        stats.count(false, Action::Stand, &ti);
        assert_eq!(1, stats.hard_count);
        assert_eq!(0, stats.hard_wrong);
        assert_eq!(0, stats.soft_count);
        assert_eq!(0, stats.split_count);
        assert_eq!(0, stats.double_count);
    }

    #[test]
    fn count_hard_wrong_increments_hard_wrong() {
        let mut stats = Stats::default();
        let ti = make_table_index(TableType::Hard, 14, 7);
        stats.count(true, Action::Hit, &ti);
        assert_eq!(1, stats.hard_count);
        assert_eq!(1, stats.hard_wrong);
    }

    // --- count(): Soft category ---

    #[test]
    fn count_soft_correct_increments_soft_count_only() {
        let mut stats = Stats::default();
        let ti = make_table_index(TableType::Soft, 17, 6);
        stats.count(false, Action::Double, &ti);
        assert_eq!(1, stats.soft_count);
        assert_eq!(0, stats.soft_wrong);
        assert_eq!(0, stats.hard_count);
        assert_eq!(0, stats.split_count);
    }

    #[test]
    fn count_soft_wrong_increments_soft_wrong() {
        let mut stats = Stats::default();
        let ti = make_table_index(TableType::Soft, 17, 6);
        stats.count(true, Action::Stand, &ti);
        assert_eq!(1, stats.soft_count);
        assert_eq!(1, stats.soft_wrong);
    }

    // --- count(): Split category ---

    #[test]
    fn count_split_correct_increments_split_count_only() {
        let mut stats = Stats::default();
        let ti = make_table_index(TableType::Split, 8, 6);
        stats.count(false, Action::Split, &ti);
        assert_eq!(1, stats.split_count);
        assert_eq!(0, stats.split_wrong);
        assert_eq!(0, stats.hard_count);
        assert_eq!(0, stats.soft_count);
    }

    #[test]
    fn count_split_wrong_increments_split_wrong() {
        let mut stats = Stats::default();
        let ti = make_table_index(TableType::Split, 8, 6);
        stats.count(true, Action::Hit, &ti);
        assert_eq!(1, stats.split_count);
        assert_eq!(1, stats.split_wrong);
    }

    // --- count(): Double is cross-cutting ---

    #[test]
    fn count_double_from_hard_increments_both_hard_and_double() {
        let mut stats = Stats::default();
        let ti = make_table_index(TableType::Hard, 11, 5);
        stats.count(false, Action::Double, &ti);
        assert_eq!(1, stats.hard_count);
        assert_eq!(0, stats.hard_wrong);
        assert_eq!(1, stats.double_count);
        assert_eq!(0, stats.double_wrong);
    }

    #[test]
    fn count_double_from_soft_wrong_increments_both_soft_and_double_wrong() {
        let mut stats = Stats::default();
        let ti = make_table_index(TableType::Soft, 18, 3);
        stats.count(true, Action::Double, &ti);
        assert_eq!(1, stats.soft_count);
        assert_eq!(1, stats.soft_wrong);
        assert_eq!(1, stats.double_count);
        assert_eq!(1, stats.double_wrong);
    }

    #[test]
    fn count_non_double_action_does_not_increment_double_count() {
        let mut stats = Stats::default();
        let ti = make_table_index(TableType::Hard, 16, 9);
        stats.count(false, Action::Stand, &ti);
        assert_eq!(0, stats.double_count);
        assert_eq!(0, stats.double_wrong);
    }

    // --- count(): Surrender maps to Hard ---

    #[test]
    fn count_surrender_table_type_maps_to_hard_bucket() {
        let mut stats = Stats::default();
        let ti = make_table_index(TableType::Surrender, 16, 9);
        stats.count(false, Action::Surrender, &ti);
        assert_eq!(1, stats.hard_count);
        assert_eq!(0, stats.hard_wrong);
        assert_eq!(0, stats.soft_count);
        assert_eq!(0, stats.split_count);
    }

    // --- count(): multiple calls accumulate correctly ---

    #[test]
    fn count_accumulates_across_multiple_calls() {
        let mut stats = Stats::default();
        let hard_ti = make_table_index(TableType::Hard, 12, 4);
        let soft_ti = make_table_index(TableType::Soft, 17, 5);
        let split_ti = make_table_index(TableType::Split, 8, 6);

        stats.count(false, Action::Stand, &hard_ti);
        stats.count(true, Action::Hit, &hard_ti);
        stats.count(false, Action::Double, &soft_ti);
        stats.count(false, Action::Split, &split_ti);

        assert_eq!(4, stats.question_count);
        assert_eq!(1, stats.questions_wrong);
        assert_eq!(2, stats.hard_count);
        assert_eq!(1, stats.hard_wrong);
        assert_eq!(1, stats.soft_count);
        assert_eq!(0, stats.soft_wrong);
        assert_eq!(1, stats.split_count);
        assert_eq!(1, stats.double_count);
        assert_eq!(0, stats.double_wrong);
    }
}
