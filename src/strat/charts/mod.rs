use crate::strat::charts::hard_chart::HardChart;
use crate::strat::charts::soft_chart::SoftChart;
use crate::strat::charts::split_chart::SplitChart;
use crate::strat::charts::surrender_chart::SurrenderChart;
use crate::strat::tableindex::ColIndex;
use crate::Action::Double;
use crate::{Action, BjResult, Hand};

mod hard_chart;
mod soft_chart;
mod split_chart;
mod surrender_chart;

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
    pub fn apply_rules(self) -> Option<Action> {
        // Currently, we have no rules.
        match self {
            ChartAction::DblH | ChartAction::DblS => Some(Double),
            ChartAction::Hit_ => Some(Action::Hit),
            ChartAction::Stnd => Some(Action::Stand),
            ChartAction::Splt => Some(Action::Split),
            ChartAction::SDas => Some(Action::Split),
            _ => {
                println!("CAN'T APPLY RULES: {:?}", self);
                None
            }
        }
    }
}

pub fn lookup_action(player_hand: &Hand, dealer_hand: &Hand) -> BjResult<ChartAction> {
    // order of ops:
    // 1. should I surrender
    // 2. should I split
    // 3. should I double
    // 4. should I hit
    // 5. stand

    let chart_action = SurrenderChart::lookup_action(player_hand, dealer_hand)?;
    if chart_action != ChartAction::NoAc {
        return Ok(chart_action);
    }

    if player_hand.splittable() {
        let chart_action = SplitChart::lookup_action(player_hand, dealer_hand)?;
        if chart_action != ChartAction::NoAc {
            return Ok(chart_action);
        }
    }

    if player_hand.is_soft() {
        SoftChart::lookup_action(player_hand, dealer_hand)
    } else {
        HardChart::lookup_action(player_hand, dealer_hand)
    }
}

trait Chart {
    fn lookup_action(player_hand: &Hand, dealer_hand: &Hand) -> BjResult<ChartAction>;
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
    use std::str::FromStr;

    pub fn lookup_test_hands<C: Chart>(player: &[&str], dealer: &[&str]) -> BjResult<ChartAction> {
        let (player_hand, dealer_hand) = make_hands(player, dealer);
        C::lookup_action(&player_hand, &dealer_hand)
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
}
