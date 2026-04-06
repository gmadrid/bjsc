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

    /// A natural blackjack: exactly two cards totaling 21.
    pub fn is_natural(&self) -> bool {
        self.cards.len() == 2 && self.total == 21
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

    fn parse(s: &str) -> Hand {
        s.parse().unwrap()
    }

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

    // --- total() ---

    #[test]
    fn total_two_non_ace_cards() {
        let h = parse("9H 8C");
        assert_eq!(17, h.total());
    }

    #[test]
    fn total_ace_counts_as_11_when_not_busting() {
        let h = parse("AS 6C");
        assert_eq!(17, h.total());
    }

    #[test]
    fn total_ace_reduced_to_1_when_would_bust() {
        let h = parse("AS 6C TC");
        // Ace + 6 + 10 = 27 -> reduce Ace: 1 + 6 + 10 = 17
        assert_eq!(17, h.total());
    }

    #[test]
    fn total_two_aces_one_soft_one_hard() {
        let h = parse("AS AC");
        // First Ace = 11, second = 1 to avoid bust: 12
        assert_eq!(12, h.total());
    }

    #[test]
    fn total_natural_blackjack() {
        let h = parse("AS TC");
        assert_eq!(21, h.total());
    }

    // --- is_soft() ---

    #[test]
    fn is_soft_true_when_ace_counts_as_11() {
        let h = parse("AS 6C");
        assert!(h.is_soft());
    }

    #[test]
    fn is_soft_false_when_ace_reduced_to_1() {
        // A + 6 + 10 = 17, Ace must be 1
        let h = parse("AS 6C TC");
        assert!(!h.is_soft());
    }

    #[test]
    fn is_soft_false_for_hard_hand() {
        let h = parse("9H 8C");
        assert!(!h.is_soft());
    }

    #[test]
    fn is_soft_true_for_blackjack() {
        let h = parse("AS TC");
        assert!(h.is_soft());
    }

    // --- is_natural() ---

    #[test]
    fn is_natural_true_for_ace_ten() {
        let h = parse("AS TC");
        assert!(h.is_natural());
    }

    #[test]
    fn is_natural_true_for_ace_face_card() {
        let h = parse("AS KC");
        assert!(h.is_natural());
    }

    #[test]
    fn is_natural_false_for_three_card_21() {
        let h = parse("7H 7C 7D");
        assert!(!h.is_natural());
    }

    #[test]
    fn is_natural_false_for_two_card_non_21() {
        let h = parse("9H 8C");
        assert!(!h.is_natural());
    }

    #[test]
    fn is_natural_false_for_empty_hand() {
        let h = Hand::default();
        assert!(!h.is_natural());
    }

    // --- splittable() ---

    #[test]
    fn splittable_true_for_two_equal_value_cards() {
        let h = parse("8H 8C");
        assert!(h.splittable());
    }

    #[test]
    fn splittable_true_for_two_aces() {
        let h = parse("AH AC");
        assert!(h.splittable());
    }

    #[test]
    fn splittable_true_for_ten_and_jack() {
        // Both have value 10
        let h = parse("TH JC");
        assert!(h.splittable());
    }

    #[test]
    fn splittable_false_for_unequal_cards() {
        let h = parse("8H 9C");
        assert!(!h.splittable());
    }

    #[test]
    fn splittable_false_for_three_cards() {
        let h = parse("5H 5C 5D");
        assert!(!h.splittable());
    }

    #[test]
    fn splittable_false_for_one_card() {
        let h = parse("8H");
        assert!(!h.splittable());
    }

    #[test]
    fn splittable_false_for_empty_hand() {
        let h = Hand::default();
        assert!(!h.splittable());
    }

    // --- first_card() ---

    #[test]
    fn first_card_returns_first_added_card() {
        let h = parse("9H 3C");
        let card = h.first_card().unwrap();
        // 9H should be the first card; value 9
        assert_eq!(9, card.value());
    }

    #[test]
    fn first_card_returns_none_for_empty_hand() {
        let h = Hand::default();
        assert!(h.first_card().is_none());
    }

    #[test]
    fn first_card_returns_ace_value_11() {
        let h = parse("AS 6C");
        let card = h.first_card().unwrap();
        assert_eq!(11, card.value()); // Ace = 11
    }

    // --- from_str / Display round-trip ---

    #[test]
    fn from_str_parses_multiple_cards() {
        let h: Hand = "9H 8C".parse().unwrap();
        assert_eq!(17, h.total());
        assert_eq!(2, h.num_cards());
    }

    #[test]
    fn from_str_invalid_card_returns_error() {
        let result: Result<Hand, _> = "XX".parse();
        assert!(result.is_err());
    }
}
