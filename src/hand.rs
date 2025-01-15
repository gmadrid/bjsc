use crate::card::{Card, Pip};
use crate::{BjError, BjResult};
use itertools::Itertools;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct Hand {
    cards: Vec<Card>,
    total: u8,
    soft: bool,
}

impl Hand {
    pub fn total(&self) -> u8 {
        self.total
    }

    pub fn is_soft(&self) -> bool {
        self.soft
    }

    pub fn first_card(&self) -> Option<Card> {
        self.cards.first().copied()
    }

    pub fn splittable(&self) -> bool {
        if self.cards.len() != 2 {
            false
        } else {
            self.cards[0].value() == self.cards[1].value()
        }
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn num_cards(&self) -> usize {
        self.cards.len()
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
        self.compute_total();
    }

    fn compute_total(&mut self) {
        let (mut hard_total, mut num_aces) = self.cards.iter().fold((0, 0), |(tot, aces), card| {
            let new_total = tot + card.value();
            let aces = aces + if card.pip == Pip::Ace { 1 } else { 0 };
            (new_total, aces)
        });

        while hard_total > 21 && num_aces > 0 {
            hard_total -= 10;
            num_aces -= 1;
        }

        self.total = hard_total;
        self.soft = num_aces > 0;
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hand_str = self.cards.iter().join(" ");
        write!(f, "{}", hand_str)
    }
}

impl FromStr for Hand {
    type Err = BjError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cards: Vec<Card> = s
            .split_whitespace()
            .map(|s| s.parse())
            .collect::<BjResult<Vec<Card>>>()?;
        let mut hand = Hand {
            cards,
            ..Default::default()
        };
        hand.compute_total();
        Ok(hand)
    }
}

#[cfg(test)]
mod tests {
    use crate::hand::Hand;

    #[test]
    fn test_add_card() {
        let mut h = Hand::default();
        h.add_card("AS".parse().unwrap());
        assert_eq!(h.total(), 11);

        h.add_card("AH".parse().unwrap());
        assert_eq!(h.total(), 12);

        h.add_card("TD".parse().unwrap());
        assert_eq!(h.total(), 12);

        h.add_card("TC".parse().unwrap());
        assert_eq!(h.total(), 22);
    }
}
