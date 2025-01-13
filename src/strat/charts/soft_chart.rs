use crate::strat::charts::ChartAction::{DblH, DblS, Hit_, Stnd};
use crate::strat::charts::{as_chart_column, Chart, ChartAction};
use crate::strat::tableindex::ColIndex;
use crate::strat::ChartAction::NoAc;
use crate::{BjError, BjResult, Hand};

// Standard Basic Strategy Soft Totals from BJA
const SOFT_CHART: [[ChartAction; 10]; 9] = [
    /* 13 (A, 2) */
    [Hit_, Hit_, Hit_, DblH, DblH, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 14 (A, 3) */
    [Hit_, Hit_, Hit_, DblH, DblH, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 15 (A, 4) */
    [Hit_, Hit_, DblH, DblH, DblH, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 16 (A, 5) */
    [Hit_, Hit_, DblH, DblH, DblH, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 17 (A, 6) */
    [Hit_, DblH, DblH, DblH, DblH, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 18 (A, 7) */
    [DblS, DblS, DblS, DblS, DblS, Stnd, Stnd, Hit_, Hit_, Hit_],
    /* 19 (A, 8) */
    [Stnd, Stnd, Stnd, Stnd, DblS, Stnd, Stnd, Stnd, Stnd, Stnd],
    /* 20 (A, 9) */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd],
    /* 21 (A, 10) */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd],
];

pub struct SoftChart;

impl Chart for SoftChart {
    fn lookup_action(player_hand: &Hand, dealer_hand: &Hand) -> BjResult<ChartAction> {
        if !player_hand.is_soft() {
            return Ok(NoAc);
        }

        let dealer_card = dealer_hand.first_card().ok_or(BjError::MissingDealerCard)?;
        let total = player_hand.total();
        if !(13..=21).contains(&total) {
            // All soft-totals *must* be in the range of 13--21.
            // (12 is only possible with A,A which is handled by the SplitChart.)
            return Err(BjError::ValueOutOfRange(total, 13, 21));
        }

        let col_index = ColIndex::new_with_card(dealer_card)?;
        let chart_index = as_chart_column(col_index);
        Ok(SOFT_CHART[(total - 13) as usize][chart_index])
    }
}

// fn lookup(index: TableIndex) -> Result<ChartAction, ()> {
//     if index.table_type() != TableType::Soft {
//         return Err(());
//     }
//
//     let row_index = index.row_index();
//     let col_index = index.col_index();
//
//     lookup_total_by_index(row_index, col_index)
// }

// fn lookup_total_by_index(total: u8, col_index: ColIndex) -> Result<ChartAction, ()> {
//     if !(13..=21).contains(&total) {
//         dbg!(total);
//         return Err(());
//     }
//
//     let chart_index = as_chart_column(col_index);
//     Ok(SOFT_CHART[(total - 13) as usize][chart_index])
// }
//
// pub fn lookup_total(total: u8, dealer: Card) -> Result<ChartAction, ()> {
//     dbg!(lookup_total_by_index(
//         total,
//         dbg!(ColIndex::new_with_card(dbg!(dealer)))?
//     ))
// }

#[cfg(test)]
mod test {
    use super::*;

    // (player_hand, dealer_hand) -> ChartAction
    const LTH_E: fn(&[&str], &[&str]) -> BjResult<ChartAction> =
        crate::strat::charts::test::lookup_test_hands::<SoftChart>;
    const LTH: fn(&[&str], &[&str]) -> ChartAction = |p, d| LTH_E(p, d).unwrap();

    #[test]
    fn test_lookup() {
        assert_eq!(Hit_, LTH(&["AH", "2C"], &["AS"]));
        assert_eq!(Hit_, LTH(&["AH", "2C"], &["2S"]));
        assert_eq!(Hit_, LTH(&["AH", "2C"], &["4S"]));
        assert_eq!(DblH, LTH(&["AH", "2C"], &["5S"]));
        assert_eq!(DblH, LTH(&["AH", "2C"], &["6S"]));
        assert_eq!(Hit_, LTH(&["AH", "2C"], &["7S"]));

        assert_eq!(Hit_, LTH(&["AH", "6C"], &["AS"]));
        assert_eq!(Hit_, LTH(&["AH", "6C"], &["2S"]));
        assert_eq!(DblH, LTH(&["AH", "6C"], &["3S"]));
        assert_eq!(DblH, LTH(&["AH", "6C"], &["5S"]));
        assert_eq!(DblH, LTH(&["AH", "6C"], &["6S"]));
        assert_eq!(Hit_, LTH(&["AH", "6C"], &["7S"]));

        assert_eq!(Hit_, LTH(&["AH", "7C"], &["AS"]));
        assert_eq!(DblS, LTH(&["AH", "7C"], &["2S"]));
        assert_eq!(DblS, LTH(&["AH", "7C"], &["3S"]));
        assert_eq!(DblS, LTH(&["AH", "7C"], &["5S"]));
        assert_eq!(DblS, LTH(&["AH", "7C"], &["6S"]));
        assert_eq!(Stnd, LTH(&["AH", "7C"], &["7S"]));
        assert_eq!(Stnd, LTH(&["AH", "7C"], &["8S"]));
        assert_eq!(Hit_, LTH(&["AH", "7C"], &["TS"]));

        assert_eq!(Stnd, LTH(&["AH", "8C"], &["AS"]));
        assert_eq!(Stnd, LTH(&["AH", "8C"], &["2S"]));
        assert_eq!(Stnd, LTH(&["AH", "8C"], &["5S"]));
        assert_eq!(DblS, LTH(&["AH", "8C"], &["6S"]));
        assert_eq!(Stnd, LTH(&["AH", "8C"], &["7S"]));
        assert_eq!(Stnd, LTH(&["AH", "8C"], &["TS"]));
    }
}
