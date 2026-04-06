use rand::prelude::*;

use crate::card::Card;

const CARDS_IN_A_DECK: usize = 52;
const PEN: usize = 26;

#[derive(Debug)]
pub struct Shoe {
    cards: Vec<Card>,
    next: usize,
    pen: usize,
}

impl Shoe {
    pub fn new(num_decks: usize) -> Shoe {
        let mut cards = Vec::with_capacity(CARDS_IN_A_DECK * num_decks);

        for _ in 0..num_decks {
            for c in 0..CARDS_IN_A_DECK {
                // unwrap: we know the indices are in range.
                cards.push((c as u8).try_into().unwrap());
            }
        }

        Shoe {
            cards,
            next: 0,
            pen: num_decks * CARDS_IN_A_DECK - PEN,
        }
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
        self.next = 0;
    }

    pub fn is_done(&self) -> bool {
        self.next >= self.pen
    }

    pub fn deal(&mut self) -> Option<Card> {
        if self.next >= self.cards.len() {
            return None;
        }
        let card = self.cards[self.next];
        self.next += 1;
        Some(card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- new() ---

    #[test]
    fn new_single_deck_has_52_cards() {
        let shoe = Shoe::new(1);
        // Deal all cards to count them
        let mut s = shoe;
        let mut count = 0usize;
        // Deal past the pen by forcing all deals
        while s.next < s.cards.len() {
            s.deal();
            count += 1;
        }
        assert_eq!(52, count);
    }

    #[test]
    fn new_six_deck_shoe_has_312_cards() {
        let shoe = Shoe::new(6);
        assert_eq!(312, shoe.cards.len());
    }

    #[test]
    fn new_two_deck_shoe_has_104_cards() {
        let shoe = Shoe::new(2);
        assert_eq!(104, shoe.cards.len());
    }

    // --- is_done() ---

    #[test]
    fn is_done_false_at_start() {
        let shoe = Shoe::new(6);
        assert!(!shoe.is_done());
    }

    #[test]
    fn is_done_true_after_dealing_past_pen() {
        let mut shoe = Shoe::new(1);
        // PEN constant is 26; a single deck shoe has 52 cards.
        // pen = 52 - 26 = 26. After dealing 26 cards, is_done() should be true.
        for _ in 0..26 {
            shoe.deal();
        }
        assert!(shoe.is_done());
    }

    #[test]
    fn is_done_false_before_pen() {
        let mut shoe = Shoe::new(1);
        // Deal one card — still well below pen
        shoe.deal();
        assert!(!shoe.is_done());
    }

    #[test]
    fn new_shoe_not_done_for_six_decks() {
        let shoe = Shoe::new(6);
        // pen = 312 - 26 = 286; clearly not done at 0
        assert!(!shoe.is_done());
    }

    // --- deal() ---

    #[test]
    fn deal_returns_some_at_start() {
        let mut shoe = Shoe::new(1);
        assert!(shoe.deal().is_some());
    }

    #[test]
    fn deal_returns_none_when_all_cards_exhausted() {
        let mut shoe = Shoe::new(1);
        // Exhaust all 52 cards
        for _ in 0..52 {
            shoe.deal();
        }
        assert!(shoe.deal().is_none());
    }

    #[test]
    fn deal_advances_position() {
        let mut shoe = Shoe::new(1);
        shoe.deal();
        shoe.deal();
        // After two deals, next should be 2
        assert_eq!(2, shoe.next);
    }

    #[test]
    fn deal_returns_cards_sequentially() {
        let mut shoe = Shoe::new(1);
        // The first two cards dealt should be the first two in the cards vector
        let expected_first = shoe.cards[0];
        let expected_second = shoe.cards[1];
        let first = shoe.deal().unwrap();
        let second = shoe.deal().unwrap();
        assert_eq!(expected_first, first);
        assert_eq!(expected_second, second);
    }

    // --- shuffle() ---

    #[test]
    fn shuffle_resets_next_to_zero() {
        let mut shoe = Shoe::new(1);
        shoe.deal();
        shoe.deal();
        assert_eq!(2, shoe.next);
        shoe.shuffle();
        assert_eq!(0, shoe.next);
    }

    #[test]
    fn shuffle_makes_shoe_not_done() {
        let mut shoe = Shoe::new(1);
        // Exhaust shoe past pen
        for _ in 0..26 {
            shoe.deal();
        }
        assert!(shoe.is_done());
        shoe.shuffle();
        assert!(!shoe.is_done());
    }

    #[test]
    fn shuffle_preserves_card_count() {
        let mut shoe = Shoe::new(6);
        let original_count = shoe.cards.len();
        shoe.shuffle();
        assert_eq!(original_count, shoe.cards.len());
    }

    // --- pen position for multi-deck shoes ---

    #[test]
    fn six_deck_shoe_pen_is_correct() {
        let shoe = Shoe::new(6);
        // pen = 312 - 26 = 286
        assert_eq!(286, shoe.pen);
    }

    #[test]
    fn one_deck_shoe_pen_is_correct() {
        let shoe = Shoe::new(1);
        // pen = 52 - 26 = 26
        assert_eq!(26, shoe.pen);
    }

    #[test]
    fn is_done_exactly_at_pen_is_true() {
        let mut shoe = Shoe::new(1);
        // Deal exactly pen (26) cards
        for _ in 0..shoe.pen {
            shoe.deal();
        }
        assert!(shoe.is_done());
    }

    #[test]
    fn is_done_one_before_pen_is_false() {
        let mut shoe = Shoe::new(1);
        let pen = shoe.pen;
        for _ in 0..(pen - 1) {
            shoe.deal();
        }
        assert!(!shoe.is_done());
    }
}
