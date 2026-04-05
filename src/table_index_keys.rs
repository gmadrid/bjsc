use crate::strat::{
    lookup_by_index, new_table_index, ChartAction, ColIndex, RowIndex, TableIndex, TableType,
};
use crate::studymode::StudyMode;

/// Enumerate all valid TableIndex cells for a given TableType.
fn indices_for_type(tt: TableType) -> Vec<TableIndex> {
    let row_range: Box<dyn Iterator<Item = u8>> = match tt {
        TableType::Hard => Box::new(8..=17),
        TableType::Soft => Box::new(13..=20), // exclude 21 (natural blackjack)
        TableType::Split => Box::new(1..=10),
        TableType::Surrender => Box::new(15..=16), // only rows with actual decisions
    };

    let mut result = Vec::new();
    for row in row_range {
        for col in 1..=10u8 {
            if let (Ok(ri), Ok(ci)) = (RowIndex::new(tt, row), col.to_string().parse::<ColIndex>())
            {
                result.push(new_table_index(ri, ci));
            }
        }
    }
    result
}

/// Get all valid TableIndex cells for a study mode.
pub fn indices_for_mode(mode: StudyMode) -> Vec<TableIndex> {
    match mode {
        StudyMode::All => {
            // All modes combined
            let mut all = Vec::new();
            all.extend(indices_for_type(TableType::Hard));
            all.extend(indices_for_type(TableType::Soft));
            all.extend(indices_for_type(TableType::Split));
            all
        }
        StudyMode::Hard => indices_for_type(TableType::Hard),
        StudyMode::Soft => indices_for_type(TableType::Soft),
        StudyMode::Splits => indices_for_type(TableType::Split),
        StudyMode::Doubles => {
            // All cells where the correct action is Double (DblH or DblS)
            let mut result = Vec::new();
            for tt in [TableType::Hard, TableType::Soft] {
                for idx in indices_for_type(tt) {
                    if let Ok(action) = lookup_by_index(&idx) {
                        if matches!(action, ChartAction::DblH | ChartAction::DblS) {
                            result.push(idx);
                        }
                    }
                }
            }
            result
        }
        StudyMode::Drill => {
            // Drill uses all questions
            let mut all = Vec::new();
            all.extend(indices_for_type(TableType::Hard));
            all.extend(indices_for_type(TableType::Soft));
            all.extend(indices_for_type(TableType::Split));
            all
        }
    }
}

/// Convert a TableIndex to a spaced-rep key string.
pub fn table_index_to_key(ti: &TableIndex) -> String {
    ti.to_string()
}

/// Get all spaced-rep keys for a study mode.
pub fn keys_for_mode(mode: StudyMode) -> Vec<String> {
    indices_for_mode(mode)
        .iter()
        .map(table_index_to_key)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hard_indices_count() {
        let indices = indices_for_type(TableType::Hard);
        assert_eq!(indices.len(), 100); // 10 rows * 10 cols
    }

    #[test]
    fn test_soft_indices_count() {
        let indices = indices_for_type(TableType::Soft);
        assert_eq!(indices.len(), 80); // 8 rows (13-20) * 10 cols, excludes natural 21
    }

    #[test]
    fn test_split_indices_count() {
        let indices = indices_for_type(TableType::Split);
        assert_eq!(indices.len(), 100); // 10 rows * 10 cols
    }

    #[test]
    fn test_doubles_are_subset() {
        let doubles = indices_for_mode(StudyMode::Doubles);
        assert!(!doubles.is_empty());
        for idx in &doubles {
            let action = lookup_by_index(idx).unwrap();
            assert!(
                matches!(action, ChartAction::DblH | ChartAction::DblS),
                "non-double action found: {:?}",
                action
            );
        }
    }

    #[test]
    fn test_key_roundtrip() {
        let indices = indices_for_mode(StudyMode::Hard);
        for idx in &indices {
            let key = table_index_to_key(idx);
            let restored: TableIndex = key.parse().unwrap();
            assert_eq!(*idx, restored);
        }
    }
}
