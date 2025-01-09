use crate::strat::tableindex::ColIndex;

mod hard_chart;
mod soft_chart;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ChartAction {
    DBLH, // Double if allowed, otherwise Hit.
    DBLS, // Double if allowed, otherwise Stand.
    HIT_,
    STND,
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
