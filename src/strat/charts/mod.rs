use crate::strat::tableindex::ColIndex;

mod hard_chart;
mod soft_chart;
mod split_chart;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ChartAction {
    DblH, // Double if allowed, otherwise Hit.
    DblS, // Double if allowed, otherwise Stand.
    Hit_,
    Stnd,

    Splt, // Split
    SDas, // Split if Double After Split allowed
    NSpt, // Don't split.
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
    use crate::strat::tableindex::TableIndex;

    pub fn ti(s: &str) -> TableIndex {
        s.parse().unwrap()
    }
}
