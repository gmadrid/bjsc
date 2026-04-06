use crate::strat::charts::hard_chart::HardChart;
use crate::strat::charts::soft_chart::SoftChart;
use crate::strat::charts::split_chart::SplitChart;
use crate::strat::charts::surrender_chart::SurrenderChart;
use crate::strat::tableindex::{ColIndex, TableIndex};
use crate::Action::Double;
use crate::{Action, BjResult, Hand};

mod hard_chart;
mod soft_chart;
mod split_chart;
mod surrender_chart;

// A list of possible values in the cells of the Basic Strategy charts.
//
// Every chart maps a players hand and the dealer's up card to an action.
// The actions in the chart often depend on the Rules or the state of the game. (E.g., you cannot
// re-split Aces, or you cannot double with three cards.) The ChartAction enum includes
// information about those contextual cues and the ultimate player action.
//
// The ChartAction needs to be resolved together with the Rules and the GameState to find
// the Player's Action.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ChartAction {
    DblH, // Double if allowed, otherwise Hit.
    DblS, // Double if allowed, otherwise Stand.
    Hit_,
    Stnd,

    Splt, // Split
    SDas, // Split if Double After Split allowed

    NoAc, // No Action
}

impl ChartAction {
    // Apply the game Rules to a ChartAction to determine the Player Action.
    pub fn apply_rules(self) -> Option<Action> {
        // Currently, we have no rules.
        match self {
            ChartAction::DblH | ChartAction::DblS => Some(Double),
            ChartAction::Hit_ => Some(Action::Hit),
            ChartAction::Stnd => Some(Action::Stand),
            ChartAction::Splt => Some(Action::Split),
            ChartAction::SDas => Some(Action::Split),
            _ => None,
        }
    }
}

// Returns a pair of the ChartAction and the index of the cell in the strategy table it came from.
// This should _never_ return ChartAction::NoAc, since there should be an Action for every valid
// (non-busted) inputs.
//
// Passing non-playable hands (because the player has busted or the dealer has started taking
// cards), will return an Error.
pub fn lookup_action(
    player_hand: &Hand,
    dealer_hand: &Hand,
) -> BjResult<(ChartAction, Option<TableIndex>)> {
    // order of ops:
    // 1. should I surrender
    // 2. should I split
    // 3. should I double
    // 4. should I hit
    // 5. stand

    let (chart_action, table_index) = SurrenderChart::lookup_action(player_hand, dealer_hand)?;
    if chart_action != ChartAction::NoAc {
        return Ok((chart_action, table_index));
    }

    if player_hand.splittable() {
        let (chart_action, table_index) = SplitChart::lookup_action(player_hand, dealer_hand)?;
        if chart_action != ChartAction::NoAc {
            return Ok((chart_action, table_index));
        }
    }

    if player_hand.is_soft() {
        SoftChart::lookup_action(player_hand, dealer_hand)
    } else {
        HardChart::lookup_action(player_hand, dealer_hand)
    }
}

/// Look up the ChartAction for a given TableIndex directly (without needing actual hands).
pub fn lookup_by_index(index: &TableIndex) -> BjResult<ChartAction> {
    let col = index.col_index();
    let chart_col = as_chart_column(col);
    let row = index.row_index();

    match index.table_type() {
        crate::strat::TableType::Hard => {
            let chart_row = if row <= 8 {
                0
            } else if row >= 17 {
                9
            } else {
                (row - 8) as usize
            };
            Ok(hard_chart::HARD_CHART[chart_row][chart_col])
        }
        crate::strat::TableType::Soft => {
            if !(13..=21).contains(&row) {
                return Err(crate::BjError::ValueOutOfRange(row, 13, 21));
            }
            Ok(soft_chart::SOFT_CHART[(row - 13) as usize][chart_col])
        }
        crate::strat::TableType::Split => {
            if !(1..=10).contains(&row) {
                return Err(crate::BjError::ValueOutOfRange(row, 1, 10));
            }
            Ok(split_chart::SPLIT_CHART[(row - 1) as usize][chart_col])
        }
        crate::strat::TableType::Surrender => {
            // Surrender chart is stubbed — always NoAc
            Ok(ChartAction::NoAc)
        }
    }
}

/// A displayable strategy chart: row labels, column headers, and cell values.
pub struct StrategyChart {
    pub title: &'static str,
    pub col_headers: Vec<&'static str>,
    pub rows: Vec<(&'static str, Vec<&'static str>)>,
}

/// Get all strategy charts for display.
pub fn all_charts() -> Vec<StrategyChart> {
    let cols = vec!["2", "3", "4", "5", "6", "7", "8", "9", "T", "A"];

    let action_str = |a: ChartAction| -> &'static str {
        match a {
            ChartAction::Hit_ => "H",
            ChartAction::Stnd => "S",
            ChartAction::DblH => "Dh",
            ChartAction::DblS => "Ds",
            ChartAction::Splt => "P",
            ChartAction::SDas => "Pd",
            ChartAction::NoAc => "-",
        }
    };

    let hard_rows: Vec<(&str, Vec<&str>)> = (0..10)
        .map(|r| {
            let label: &'static str = match r {
                0 => "8",
                1 => "9",
                2 => "10",
                3 => "11",
                4 => "12",
                5 => "13",
                6 => "14",
                7 => "15",
                8 => "16",
                _ => "17",
            };
            let cells: Vec<&str> = (0..10)
                .map(|c| action_str(hard_chart::HARD_CHART[r][c]))
                .collect();
            (label, cells)
        })
        .collect();

    let soft_rows: Vec<(&str, Vec<&str>)> = (0..9)
        .map(|r| {
            let label: &'static str = match r {
                0 => "A,2",
                1 => "A,3",
                2 => "A,4",
                3 => "A,5",
                4 => "A,6",
                5 => "A,7",
                6 => "A,8",
                7 => "A,9",
                _ => "A,T",
            };
            let cells: Vec<&str> = (0..10)
                .map(|c| action_str(soft_chart::SOFT_CHART[r][c]))
                .collect();
            (label, cells)
        })
        .collect();

    let split_rows: Vec<(&str, Vec<&str>)> = (0..10)
        .map(|r| {
            let label: &'static str = match r {
                0 => "A,A",
                1 => "2,2",
                2 => "3,3",
                3 => "4,4",
                4 => "5,5",
                5 => "6,6",
                6 => "7,7",
                7 => "8,8",
                8 => "9,9",
                _ => "T,T",
            };
            let cells: Vec<&str> = (0..10)
                .map(|c| action_str(split_chart::SPLIT_CHART[r][c]))
                .collect();
            (label, cells)
        })
        .collect();

    vec![
        StrategyChart {
            title: "Hard Totals",
            col_headers: cols.clone(),
            rows: hard_rows,
        },
        StrategyChart {
            title: "Soft Totals",
            col_headers: cols.clone(),
            rows: soft_rows,
        },
        StrategyChart {
            title: "Pairs (Split)",
            col_headers: cols,
            rows: split_rows,
        },
    ]
}

trait Chart {
    fn lookup_action(
        player_hand: &Hand,
        dealer_hand: &Hand,
    ) -> BjResult<(ChartAction, Option<TableIndex>)>;
}

// Chart columns are 2-9, A.
// ColIndex is 1-10. (1 = Ace)
// This function maps ColIndex into chart columns.
fn as_chart_column(ci: ColIndex) -> usize {
    let val = ci.value();
    if val == 1 {
        9
    } else {
        (val - 2) as usize
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::card::Card;
    use crate::strat::{new_table_index, ColIndex, RowIndex, TableType};
    use std::str::FromStr;

    pub fn lookup_test_hands<C: Chart>(player: &[&str], dealer: &[&str]) -> BjResult<ChartAction> {
        let (player_hand, dealer_hand) = make_hands(player, dealer);
        let (action, _) = C::lookup_action(&player_hand, &dealer_hand)?;
        Ok(action)
    }

    // Returns (player_hand, dealer_hand).
    pub fn make_hands(player: &[&str], dealer: &[&str]) -> (Hand, Hand) {
        let dealer_hand = arr_to_hand(dealer);
        let player_hand = arr_to_hand(player);
        (player_hand, dealer_hand)
    }

    fn arr_to_hand(arr: &[&str]) -> Hand {
        arr.iter().fold(Hand::default(), |mut hand: Hand, s| {
            let card = Card::from_str(s).unwrap();
            hand.add_card(card);
            hand
        })
    }

    fn make_index(tt: TableType, row: u8, col: u8) -> crate::strat::TableIndex {
        let ri = RowIndex::new(tt, row).unwrap();
        let ci: ColIndex = col.to_string().parse().unwrap();
        new_table_index(ri, ci)
    }

    // --- ChartAction::apply_rules ---

    #[test]
    fn apply_rules_dblh_returns_double() {
        assert_eq!(Some(Action::Double), ChartAction::DblH.apply_rules());
    }

    #[test]
    fn apply_rules_dbls_returns_double() {
        assert_eq!(Some(Action::Double), ChartAction::DblS.apply_rules());
    }

    #[test]
    fn apply_rules_hit_returns_hit() {
        assert_eq!(Some(Action::Hit), ChartAction::Hit_.apply_rules());
    }

    #[test]
    fn apply_rules_stand_returns_stand() {
        assert_eq!(Some(Action::Stand), ChartAction::Stnd.apply_rules());
    }

    #[test]
    fn apply_rules_splt_returns_split() {
        assert_eq!(Some(Action::Split), ChartAction::Splt.apply_rules());
    }

    #[test]
    fn apply_rules_sdas_returns_split() {
        assert_eq!(Some(Action::Split), ChartAction::SDas.apply_rules());
    }

    #[test]
    fn apply_rules_noac_returns_none() {
        assert_eq!(None, ChartAction::NoAc.apply_rules());
    }

    // --- lookup_action: correct strategy decisions ---

    #[test]
    fn lookup_action_hard_16_vs_7_is_hit() {
        let (p, d) = make_hands(&["9H", "7C"], &["7S"]);
        let (action, _) = lookup_action(&p, &d).unwrap();
        assert_eq!(ChartAction::Hit_, action);
    }

    #[test]
    fn lookup_action_hard_17_vs_6_is_stand() {
        let (p, d) = make_hands(&["9H", "8C"], &["6S"]);
        let (action, _) = lookup_action(&p, &d).unwrap();
        assert_eq!(ChartAction::Stnd, action);
    }

    #[test]
    fn lookup_action_hard_11_vs_5_is_double() {
        let (p, d) = make_hands(&["5H", "6C"], &["5S"]);
        let (action, _) = lookup_action(&p, &d).unwrap();
        assert_eq!(ChartAction::DblH, action);
    }

    #[test]
    fn lookup_action_aces_pair_vs_6_is_split() {
        let (p, d) = make_hands(&["AH", "AC"], &["6S"]);
        let (action, _) = lookup_action(&p, &d).unwrap();
        // Aces always split
        assert_eq!(ChartAction::Splt, action);
    }

    #[test]
    fn lookup_action_eights_pair_vs_9_is_split() {
        let (p, d) = make_hands(&["8H", "8C"], &["9S"]);
        let (action, _) = lookup_action(&p, &d).unwrap();
        assert_eq!(ChartAction::Splt, action);
    }

    #[test]
    fn lookup_action_soft_18_vs_2_is_double() {
        let (p, d) = make_hands(&["AH", "7C"], &["2S"]);
        let (action, _) = lookup_action(&p, &d).unwrap();
        assert_eq!(ChartAction::DblS, action);
    }

    #[test]
    fn lookup_action_soft_18_vs_7_is_stand() {
        let (p, d) = make_hands(&["AH", "7C"], &["7S"]);
        let (action, _) = lookup_action(&p, &d).unwrap();
        assert_eq!(ChartAction::Stnd, action);
    }

    #[test]
    fn lookup_action_returns_table_index() {
        let (p, d) = make_hands(&["9H", "8C"], &["6S"]);
        let (_, idx) = lookup_action(&p, &d).unwrap();
        assert!(idx.is_some());
    }

    #[test]
    fn lookup_action_error_on_missing_dealer_card() {
        let player: Hand = "9H 8C".parse().unwrap();
        let dealer = Hand::default(); // empty
        assert!(lookup_action(&player, &dealer).is_err());
    }

    // --- lookup_by_index ---

    #[test]
    fn lookup_by_index_hard_8_is_hit() {
        let idx = make_index(TableType::Hard, 8, 5);
        assert_eq!(ChartAction::Hit_, lookup_by_index(&idx).unwrap());
    }

    #[test]
    fn lookup_by_index_hard_17_is_stand() {
        let idx = make_index(TableType::Hard, 17, 7);
        assert_eq!(ChartAction::Stnd, lookup_by_index(&idx).unwrap());
    }

    #[test]
    fn lookup_by_index_hard_11_vs_ace_is_double() {
        // Dealer Ace = col 1
        let idx = make_index(TableType::Hard, 11, 1);
        assert_eq!(ChartAction::DblH, lookup_by_index(&idx).unwrap());
    }

    #[test]
    fn lookup_by_index_soft_18_vs_2_is_double() {
        // Soft 18 (A,7) vs dealer 2
        let idx = make_index(TableType::Soft, 18, 2);
        assert_eq!(ChartAction::DblS, lookup_by_index(&idx).unwrap());
    }

    #[test]
    fn lookup_by_index_soft_20_vs_any_is_stand() {
        for col in 1u8..=10 {
            let idx = make_index(TableType::Soft, 20, col);
            assert_eq!(ChartAction::Stnd, lookup_by_index(&idx).unwrap());
        }
    }

    #[test]
    fn lookup_by_index_split_aces_always_split() {
        // Row 1 = Aces
        for col in 1u8..=10 {
            let idx = make_index(TableType::Split, 1, col);
            assert_eq!(ChartAction::Splt, lookup_by_index(&idx).unwrap());
        }
    }

    #[test]
    fn lookup_by_index_split_tens_never_split() {
        // Row 10 = tens
        for col in 1u8..=10 {
            let idx = make_index(TableType::Split, 10, col);
            assert_eq!(ChartAction::NoAc, lookup_by_index(&idx).unwrap());
        }
    }

    #[test]
    fn lookup_by_index_surrender_always_no_action() {
        let idx = make_index(TableType::Surrender, 16, 9);
        assert_eq!(ChartAction::NoAc, lookup_by_index(&idx).unwrap());
    }

    #[test]
    fn lookup_by_index_soft_valid_range_13_to_21() {
        // Confirm all valid soft rows (13-21) look up without error
        for row in 13u8..=21 {
            let idx = make_index(TableType::Soft, row, 5);
            assert!(
                lookup_by_index(&idx).is_ok(),
                "lookup_by_index failed for soft:{}",
                row
            );
        }
    }
}
