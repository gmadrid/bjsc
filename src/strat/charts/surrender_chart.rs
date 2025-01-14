use crate::strat::charts::ChartAction::NoAc;
use crate::strat::charts::{Chart, ChartAction};
use crate::strat::tableindex::TableIndex;
use crate::{BjResult, Hand};

pub struct SurrenderChart;

impl Chart for SurrenderChart {
    fn lookup_action(
        _player_hand: &Hand,
        _dealer_hand: &Hand,
    ) -> BjResult<(ChartAction, Option<TableIndex>)> {
        Ok((NoAc, None))
    }
}
