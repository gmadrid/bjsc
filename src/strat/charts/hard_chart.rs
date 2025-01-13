use crate::strat::charts::ChartAction::{DblH, Hit_, Stnd};
use crate::strat::charts::{as_chart_column, Chart, ChartAction};
use crate::strat::tableindex::ColIndex;
use crate::{BjError, BjResult, Hand};

// Standard Basic Strategy Hard Totals from BJA
const HARD_CHART: [[ChartAction; 10]; 10] = [
    /* 8 and lower */
    [Hit_, Hit_, Hit_, Hit_, Hit_, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 9 */
    [Hit_, DblH, DblH, DblH, DblH, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 10 */
    [DblH, DblH, DblH, DblH, DblH, DblH, DblH, DblH, Hit_, Hit_],
    /* 11 */
    [DblH, DblH, DblH, DblH, DblH, DblH, DblH, DblH, DblH, DblH],
    /* 12 */
    [Hit_, Hit_, Stnd, Stnd, Stnd, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 13 */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 14 */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 15 */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 16 */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 17+ */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd],
];

pub struct HardChart;

impl Chart for HardChart {
    fn lookup_action(player_hand: &Hand, dealer_hand: &Hand) -> BjResult<ChartAction> {
        let dealer_card = dealer_hand.first_card().ok_or(BjError::MissingDealerCard)?;
        let total = player_hand.total();
        let col_index = ColIndex::new_with_card(dealer_card)?;
        let chart_index = as_chart_column(col_index);

        if total <= 8 {
            Ok(HARD_CHART[0][chart_index])
        } else if total >= 17 {
            Ok(HARD_CHART[9][chart_index])
        } else {
            Ok(HARD_CHART[(total - 8) as usize][chart_index])
        }
    }
}

// fn lookup(index: TableIndex) -> Result<ChartAction, ()> {
//     if index.table_type() != TableType::Hard {
//         return Err(());
//     }
//
//     let row_index = index.row_index();
//     let col_index = index.col_index();
//     lookup_total_by_index(row_index, col_index)
// }

// fn lookup_total(total: u8, dealer: Card) -> BjResult<ChartAction> {
//     lookup_total_by_index(total, ColIndex::new_with_card(dealer)?)
// }
//
// fn lookup_total_by_index(total: u8, col_index: ColIndex) -> BjResult<ChartAction> {
//     let chart_index = as_chart_column(col_index);
//
//     if total <= 8 {
//         Ok(HARD_CHART[0][chart_index])
//     } else if total >= 17 {
//         Ok(HARD_CHART[9][chart_index])
//     } else {
//         Ok(HARD_CHART[(total - 8) as usize][chart_index])
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    // (player_hand, dealer_hand) -> ChartAction
    const LTH_E: fn(&[&str], &[&str]) -> BjResult<ChartAction> =
        crate::strat::charts::test::lookup_test_hands::<HardChart>;
    const LTH: fn(&[&str], &[&str]) -> ChartAction = |p, d| LTH_E(p, d).unwrap();

    #[test]
    fn test_lookup_low() {
        // Hit at or below 8
        assert_eq!(Hit_, LTH(&["2H", "2C"], &["AH"]));
        assert_eq!(Hit_, LTH(&["2H", "3C"], &["2H"]));
        assert_eq!(Hit_, LTH(&["4H", "2C"], &["3H"]));
        assert_eq!(Hit_, LTH(&["3H", "4C"], &["7H"]));
        assert_eq!(Hit_, LTH(&["3H", "5C"], &["TH"]));
    }

    #[test]
    fn test_lookup_mid() {
        // We are spot-checking some key values to try to check the chart lookup.
        assert_eq!(Hit_, LTH(&["5H", "4C"], &["2C"]));
        assert_eq!(DblH, LTH(&["3H", "6C"], &["3C"]));
        assert_eq!(DblH, LTH(&["2H", "7C"], &["6C"]));
        assert_eq!(Hit_, LTH(&["7H", "2C"], &["7C"]));
        assert_eq!(Hit_, LTH(&["5H", "4C"], &["AC"]));

        assert_eq!(DblH, LTH(&["5H", "6C"], &["2C"]));
        assert_eq!(Hit_, LTH(&["8H", "4C"], &["2C"]));
        assert_eq!(Stnd, LTH(&["5H", "8C"], &["2C"]));

        assert_eq!(DblH, LTH(&["5H", "6C"], &["3C"]));
        assert_eq!(Hit_, LTH(&["8H", "4C"], &["3C"]));
        assert_eq!(Stnd, LTH(&["5H", "8C"], &["3C"]));

        assert_eq!(DblH, LTH(&["5H", "6C"], &["4C"]));
        assert_eq!(Stnd, LTH(&["8H", "4C"], &["4C"]));
        assert_eq!(Stnd, LTH(&["5H", "8C"], &["4C"]));

        assert_eq!(DblH, LTH(&["5H", "6C"], &["6C"]));
        assert_eq!(Stnd, LTH(&["8H", "4C"], &["6C"]));
        assert_eq!(Stnd, LTH(&["5H", "8C"], &["6C"]));

        assert_eq!(DblH, LTH(&["5H", "6C"], &["7C"]));
        assert_eq!(Hit_, LTH(&["8H", "4C"], &["7C"]));
        assert_eq!(Hit_, LTH(&["5H", "8C"], &["7C"]));

        assert_eq!(DblH, LTH(&["5H", "6C"], &["9C"]));
        assert_eq!(Hit_, LTH(&["8H", "4C"], &["9C"]));
        assert_eq!(Hit_, LTH(&["5H", "8C"], &["9C"]));

        assert_eq!(Hit_, LTH(&["4H", "6C"], &["TC"]));
        assert_eq!(DblH, LTH(&["8H", "3C"], &["TC"]));
        assert_eq!(Hit_, LTH(&["5H", "7C"], &["TC"]));

        assert_eq!(Hit_, LTH(&["4H", "6C"], &["AC"]));
        assert_eq!(DblH, LTH(&["8H", "3C"], &["AC"]));
        assert_eq!(Hit_, LTH(&["5H", "7C"], &["AC"]));
        assert_eq!(Hit_, LTH(&["4H", "9C"], &["AC"]));
    }

    #[test]
    fn test_lookup_high() {
        // Stand at or above 17
        assert_eq!(Stnd, LTH(&["8H", "9D"], &["AC"]));
        assert_eq!(Stnd, LTH(&["8H", "TD"], &["3C"]));
        assert_eq!(Stnd, LTH(&["TH", "9D"], &["7C"]));
        assert_eq!(Stnd, LTH(&["TH", "TD"], &["9C"]));
        assert_eq!(Stnd, LTH(&["TC", "AH"], &["10C"]));
    }
}
