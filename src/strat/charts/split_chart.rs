use crate::strat::charts::ChartAction::{NoAc, SDas, Splt};
use crate::strat::charts::{as_chart_column, Chart, ChartAction};
use crate::strat::tableindex::TableType::Split;
use crate::strat::tableindex::{new_table_index, ColIndex, RowIndex, TableIndex};
use crate::{BjError, BjResult, Hand};

// Standard Basic Strategy Pair Splitting from BJA
//
// Col Index: all tables are indexed by dealer card _value_: 2-T, A.
// The Row Index for the Split Chart is interpreted as the Card in the Split: A-T.
// Note that the rows are _not_ zero-indexed.
// Note that the Row and Col indices are in a different order.
// Our code is the same as the layout of the Chart on the BJA site.
const SPLIT_CHART: [[ChartAction; 10]; 10] = [
    /*  2 (A, A) */
    [Splt, Splt, Splt, Splt, Splt, Splt, Splt, Splt, Splt, Splt],
    /*  4 (2, 2) */
    [SDas, SDas, Splt, Splt, Splt, Splt, NoAc, NoAc, NoAc, NoAc],
    /*  6 (3, 3) */
    [SDas, SDas, Splt, Splt, Splt, Splt, NoAc, NoAc, NoAc, NoAc],
    /*  8 (4, 4) */
    [NoAc, NoAc, NoAc, SDas, SDas, NoAc, NoAc, NoAc, NoAc, NoAc],
    /* 10 (5, 5) */
    [NoAc, NoAc, NoAc, NoAc, NoAc, NoAc, NoAc, NoAc, NoAc, NoAc],
    /* 12 (6, 6) */
    [SDas, Splt, Splt, Splt, Splt, NoAc, NoAc, NoAc, NoAc, NoAc],
    /* 14 (7, 7) */
    [Splt, Splt, Splt, Splt, Splt, Splt, NoAc, NoAc, NoAc, NoAc],
    /* 16 (8, 8) */
    [Splt, Splt, Splt, Splt, Splt, Splt, Splt, Splt, Splt, Splt],
    /* 18 (9, 9) */
    [Splt, Splt, Splt, Splt, Splt, NoAc, Splt, Splt, NoAc, NoAc],
    /* 20 (T, T) */
    [NoAc, NoAc, NoAc, NoAc, NoAc, NoAc, NoAc, NoAc, NoAc, NoAc],
];

pub struct SplitChart;

impl Chart for SplitChart {
    fn lookup_action(
        player_hand: &Hand,
        dealer_hand: &Hand,
    ) -> BjResult<(ChartAction, Option<TableIndex>)> {
        let dealer_card = dealer_hand.first_card().ok_or(BjError::MissingDealerCard)?;
        if !player_hand.splittable() {
            return Ok((NoAc, None));
        }

        // `splittable()` checks that the two player cards are the same.
        let col_index = ColIndex::new_with_card(dealer_card)?;
        let chart_index = as_chart_column(col_index);
        // unwrap: it's splittable, so this will work
        let row = player_hand
            .first_card()
            .map(|c| c.value())
            // An Ace has a value of 11, but it's row 1 in the chart.
            .map(|v| if v == 11 { 1 } else { v })
            .unwrap();

        let row_index = RowIndex::new(Split, row)?;
        let table_index = new_table_index(row_index, col_index);
        let chart_action = SPLIT_CHART[(row - 1) as usize][chart_index];
        Ok((chart_action, Some(table_index)))
    }
}

// fn lookup(index: TableIndex) -> Result<ChartAction, ()> {
//     if index.table_type() != TableType::Split {
//         return Err(());
//     }
//
//     let row_index = index.row_index();
//     let col_index = index.col_index();
//     lookup_total_by_index(row_index, col_index)
// }

// pub fn lookup_total(total: u8, dealer: Card) -> Result<ChartAction, ()> {
//     lookup_total_by_index(total, ColIndex::new_with_card(dealer)?)
// }
//
// fn lookup_total_by_index(total: u8, col_index: ColIndex) -> Result<ChartAction, ()> {
//     let chart_index = as_chart_column(col_index);
//
//     if total % 2 != 0 {
//         return Err(());
//     }
//
//     Ok(SPLIT_CHART[(total / 2 - 1) as usize][chart_index])
// }

#[cfg(test)]
mod test {
    use super::*;

    // (player_hand, dealer_hand) -> ChartAction
    const LTH_E: fn(&[&str], &[&str]) -> BjResult<ChartAction> =
        crate::strat::charts::test::lookup_test_hands::<SplitChart>;
    const LTH: fn(&[&str], &[&str]) -> ChartAction = |p, d| LTH_E(p, d).unwrap();

    #[test]
    fn test_lookup() {
        assert_eq!(SDas, LTH(&["2H", "2C"], &["2S"]));
        assert_eq!(SDas, LTH(&["2H", "2C"], &["3S"]));
        assert_eq!(Splt, LTH(&["2H", "2C"], &["4S"]));
        assert_eq!(Splt, LTH(&["2H", "2C"], &["7S"]));
        assert_eq!(NoAc, LTH(&["2H", "2C"], &["8S"]));
        assert_eq!(NoAc, LTH(&["2H", "2C"], &["TS"]));
        assert_eq!(NoAc, LTH(&["2H", "2C"], &["AS"]));

        assert_eq!(NoAc, LTH(&["4H", "4C"], &["2S"]));
        assert_eq!(NoAc, LTH(&["4H", "4C"], &["4S"]));
        assert_eq!(SDas, LTH(&["4H", "4C"], &["5S"]));
        assert_eq!(SDas, LTH(&["4H", "4C"], &["6S"]));
        assert_eq!(NoAc, LTH(&["4H", "4C"], &["7S"]));
        assert_eq!(NoAc, LTH(&["4H", "4C"], &["TS"]));
        assert_eq!(NoAc, LTH(&["4H", "4C"], &["AS"]));

        assert_eq!(Splt, LTH(&["9H", "9C"], &["2S"]));
        assert_eq!(Splt, LTH(&["9H", "9C"], &["6S"]));
        assert_eq!(NoAc, LTH(&["9H", "9C"], &["7S"]));
        assert_eq!(Splt, LTH(&["9H", "9C"], &["8S"]));
        assert_eq!(Splt, LTH(&["9H", "9C"], &["9S"]));
        assert_eq!(NoAc, LTH(&["9H", "9C"], &["TS"]));
        assert_eq!(NoAc, LTH(&["9H", "9C"], &["AS"]));

        assert_eq!(Splt, LTH(&["AH", "AC"], &["2S"]));
        assert_eq!(Splt, LTH(&["AH", "AC"], &["6S"]));
        assert_eq!(Splt, LTH(&["AH", "AC"], &["7S"]));
        assert_eq!(Splt, LTH(&["AH", "AC"], &["8S"]));
        assert_eq!(Splt, LTH(&["AH", "AC"], &["9S"]));
        assert_eq!(Splt, LTH(&["AH", "AC"], &["TS"]));
        assert_eq!(Splt, LTH(&["AH", "AC"], &["AS"]));
    }
}
