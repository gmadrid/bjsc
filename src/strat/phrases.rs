use crate::card::Pip::{Ace, Eight, Five, Four, Nine, Seven, Six, Ten, Three, Two};
use crate::rowidx;
use crate::strat::tableindex::RowIndex;
use crate::strat::tableindex::TableType::{Hard, Soft, Split, Surrender};
use std::collections::HashMap;
use std::sync::LazyLock;

macro_rules! phrase_row {
    ($m:expr, $t:expr, $c:expr, $p:expr) => {
        $m.insert(rowidx!($t, $c), $p);
    };
}

// These phrases are from the Blackjack Apprentice site.
// They are available at this publicly readable link: https://www.blackjackapprenticeship.com/blackjack-strategy-charts/
static PHRASES: LazyLock<HashMap<RowIndex, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    phrase_row!(
        m,
        Surrender,
        15,
        "15 surrenders against dealer 10, otherwise don’t surrender (revert to hard totals)."
    );
    phrase_row!(m, Surrender, 16, "16 surrenders against dealer 9 through Ace, otherwise don’t surrender (revert to hard totals).");

    phrase_row!(m, Split, Ace.value() - 10, "Always split Aces.");
    phrase_row!(
        m,
        Split,
        Two.value(),
        "A pair of 2’s splits against dealer 2 through 7, otherwise hit."
    );
    phrase_row!(
        m,
        Split,
        Three.value(),
        "A pair of 3’s splits against dealer 2 through 7, otherwise hit."
    );
    phrase_row!(
        m,
        Split,
        Four.value(),
        "A pair of 4’s splits against dealer 5 and 6 , otherwise hit."
    );
    phrase_row!(
        m,
        Split,
        Five.value(),
        "A pair of 5’s doubles against dealer 2 through 9 otherwise hit."
    );
    phrase_row!(
        m,
        Split,
        Six.value(),
        "A pair of 6’s splits against dealer 2 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Split,
        Seven.value(),
        "A pair of 7’s splits against dealer 2 through 7, otherwise hit."
    );
    phrase_row!(m, Split, Eight.value(), "Always split 8’s.");
    phrase_row!(
        m,
        Split,
        Nine.value(),
        "A pair of 9’s splits against dealer 2 through 9, except for 7, otherwise stand."
    );
    phrase_row!(m, Split, Ten.value(), "Never split tens.");

    phrase_row!(
        m,
        Soft,
        13,
        "Soft 13 (A,2) doubles against dealer 5 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Soft,
        14,
        "Soft 14 (A,3) doubles against dealer 5 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Soft,
        15,
        "Soft 15 (A,4) doubles against dealer 4 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Soft,
        16,
        "Soft 16 (A,5) doubles against dealer 4 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Soft,
        17,
        "Soft 17 (A,6) doubles against dealer 3 through 6, otherwise hit."
    );
    phrase_row!(m, Soft, 18, "Soft 18 (A,7) doubles against dealer 2 through 6, and hits against 9 through Ace, otherwise stand.");
    phrase_row!(
        m,
        Soft,
        19,
        "Soft 19 (A,8) doubles against dealer 6, otherwise stand."
    );
    phrase_row!(m, Soft, 20, "Soft 20 (A,9) always stands");
    phrase_row!(m, Soft, 21, "Is a Blackjack. Stand, idiot!");

    phrase_row!(m, Hard, 8, "8 always hits.");
    phrase_row!(
        m,
        Hard,
        9,
        "9 doubles against dealer 3 through 6 otherwise hit."
    );
    phrase_row!(
        m,
        Hard,
        10,
        "10 doubles against dealer 2 through 9 otherwise hit."
    );
    phrase_row!(m, Hard, 11, "11 always doubles.");
    phrase_row!(
        m,
        Hard,
        12,
        "12 stands against dealer 4 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Hard,
        13,
        "13 stands against dealer 2 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Hard,
        14,
        "14 stands against dealer 2 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Hard,
        15,
        "15 stands against dealer 2 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Hard,
        16,
        "16 stands against dealer 2 through 6, otherwise hit."
    );
    phrase_row!(m, Hard, 17, "17 and up always stands.");

    m
});

pub fn phrase_for_row(ri: RowIndex) -> &'static str {
    PHRASES
        .get(&ri)
        .unwrap_or(&"Internal Error: RowIndex unrecognized.")
}

/// All strategy phrases grouped by category, in display order.
/// Returns Vec of (category_name, Vec<phrase>).
pub fn all_phrases() -> Vec<(&'static str, Vec<&'static str>)> {
    let mut result = Vec::new();

    let surrender: Vec<&str> = [15u8, 16]
        .iter()
        .filter_map(|&r| RowIndex::new(Surrender, r).ok())
        .map(phrase_for_row)
        .collect();
    result.push(("Surrender", surrender));

    let split: Vec<&str> = [
        Ace.value() - 10,
        Two.value(),
        Three.value(),
        Four.value(),
        Five.value(),
        Six.value(),
        Seven.value(),
        Eight.value(),
        Nine.value(),
        Ten.value(),
    ]
    .iter()
    .filter_map(|&r| RowIndex::new(Split, r).ok())
    .map(phrase_for_row)
    .collect();
    result.push(("Splits", split));

    let soft: Vec<&str> = (13..=21)
        .filter_map(|r| RowIndex::new(Soft, r).ok())
        .map(phrase_for_row)
        .collect();
    result.push(("Soft Totals", soft));

    let hard: Vec<&str> = (8..=17)
        .filter_map(|r| RowIndex::new(Hard, r).ok())
        .map(phrase_for_row)
        .collect();
    result.push(("Hard Totals", hard));

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- phrase_for_row(): returns non-error strings for all known rows ---

    #[test]
    fn phrase_for_row_hard_8_through_17_all_return_real_phrases() {
        for row in 8u8..=17 {
            let ri = RowIndex::new(Hard, row).unwrap();
            let phrase = phrase_for_row(ri);
            assert!(
                !phrase.starts_with("Internal Error"),
                "unexpected error phrase for hard:{}: {}",
                row,
                phrase
            );
            assert!(!phrase.is_empty());
        }
    }

    #[test]
    fn phrase_for_row_soft_13_through_21_all_return_real_phrases() {
        for row in 13u8..=21 {
            let ri = RowIndex::new(Soft, row).unwrap();
            let phrase = phrase_for_row(ri);
            assert!(
                !phrase.starts_with("Internal Error"),
                "unexpected error phrase for soft:{}: {}",
                row,
                phrase
            );
            assert!(!phrase.is_empty());
        }
    }

    #[test]
    fn phrase_for_row_split_1_through_10_all_return_real_phrases() {
        for row in 1u8..=10 {
            let ri = RowIndex::new(Split, row).unwrap();
            let phrase = phrase_for_row(ri);
            assert!(
                !phrase.starts_with("Internal Error"),
                "unexpected error phrase for split:{}: {}",
                row,
                phrase
            );
            assert!(!phrase.is_empty());
        }
    }

    #[test]
    fn phrase_for_row_surrender_15_and_16_return_real_phrases() {
        for row in [15u8, 16] {
            let ri = RowIndex::new(Surrender, row).unwrap();
            let phrase = phrase_for_row(ri);
            assert!(
                !phrase.starts_with("Internal Error"),
                "unexpected error phrase for surrender:{}: {}",
                row,
                phrase
            );
            assert!(!phrase.is_empty());
        }
    }

    #[test]
    fn phrase_for_row_unknown_row_index_returns_internal_error() {
        // RowIndex::new validates the range, so build one for a valid row but
        // a type that has no phrase entry. Use Hard:2 which is valid for the table
        // type range check but absent from PHRASES.
        let ri = RowIndex::new(Hard, 2).unwrap();
        let phrase = phrase_for_row(ri);
        assert!(
            phrase.starts_with("Internal Error"),
            "expected internal error phrase, got: {}",
            phrase
        );
    }

    // --- specific phrase content spot-checks ---

    #[test]
    fn phrase_for_row_hard_11_mentions_always_doubles() {
        let ri = RowIndex::new(Hard, 11).unwrap();
        let phrase = phrase_for_row(ri);
        assert!(
            phrase.contains("always") || phrase.contains("double") || phrase.contains("Double"),
            "unexpected phrase for hard:11: {}",
            phrase
        );
    }

    #[test]
    fn phrase_for_row_split_aces_mentions_split() {
        let ri = RowIndex::new(Split, 1).unwrap();
        let phrase = phrase_for_row(ri);
        assert!(
            phrase.contains("split") || phrase.contains("Split"),
            "unexpected phrase for split:1: {}",
            phrase
        );
    }

    // --- all_phrases(): structure ---

    #[test]
    fn all_phrases_returns_four_categories() {
        let phrases = all_phrases();
        assert_eq!(4, phrases.len());
    }

    #[test]
    fn all_phrases_category_names_are_correct() {
        let phrases = all_phrases();
        let names: Vec<&str> = phrases.iter().map(|(name, _)| *name).collect();
        assert!(names.contains(&"Surrender"));
        assert!(names.contains(&"Splits"));
        assert!(names.contains(&"Soft Totals"));
        assert!(names.contains(&"Hard Totals"));
    }

    #[test]
    fn all_phrases_surrender_has_two_entries() {
        let phrases = all_phrases();
        let surrender = phrases
            .iter()
            .find(|(name, _)| *name == "Surrender")
            .unwrap();
        assert_eq!(2, surrender.1.len());
    }

    #[test]
    fn all_phrases_splits_has_ten_entries() {
        let phrases = all_phrases();
        let splits = phrases.iter().find(|(name, _)| *name == "Splits").unwrap();
        assert_eq!(10, splits.1.len());
    }

    #[test]
    fn all_phrases_soft_totals_has_nine_entries() {
        let phrases = all_phrases();
        let soft = phrases
            .iter()
            .find(|(name, _)| *name == "Soft Totals")
            .unwrap();
        // 13..=21 = 9 rows
        assert_eq!(9, soft.1.len());
    }

    #[test]
    fn all_phrases_hard_totals_has_ten_entries() {
        let phrases = all_phrases();
        let hard = phrases
            .iter()
            .find(|(name, _)| *name == "Hard Totals")
            .unwrap();
        // 8..=17 = 10 rows
        assert_eq!(10, hard.1.len());
    }

    #[test]
    fn all_phrases_no_entry_starts_with_internal_error() {
        let phrases = all_phrases();
        for (cat, entries) in &phrases {
            for phrase in entries {
                assert!(
                    !phrase.starts_with("Internal Error"),
                    "Internal Error phrase found in category '{}': {}",
                    cat,
                    phrase
                );
            }
        }
    }

    #[test]
    fn all_phrases_no_entry_is_empty() {
        let phrases = all_phrases();
        for (cat, entries) in &phrases {
            for phrase in entries {
                assert!(
                    !phrase.is_empty(),
                    "Empty phrase found in category '{}'",
                    cat
                );
            }
        }
    }
}
