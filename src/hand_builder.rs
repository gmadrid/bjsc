use crate::card::{Card, Pip, Suit};
use crate::hand::Hand;
use crate::strat::{TableIndex, TableType};
use rand::prelude::*;

const SUITS: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

fn random_suit() -> Suit {
    SUITS[thread_rng().gen_range(0..4)]
}

/// Map a card value (1-10, where 1=Ace) to a Pip.
/// For value 10, randomly picks Ten/Jack/Queen/King.
fn pip_for_value(val: u8) -> Pip {
    match val {
        1 => Pip::Ace,
        2 => Pip::Two,
        3 => Pip::Three,
        4 => Pip::Four,
        5 => Pip::Five,
        6 => Pip::Six,
        7 => Pip::Seven,
        8 => Pip::Eight,
        9 => Pip::Nine,
        10 => {
            let faces = [Pip::Ten, Pip::Jack, Pip::Queen, Pip::King];
            faces[thread_rng().gen_range(0..4)]
        }
        11 => Pip::Ace,
        _ => unreachable!("invalid card value: {}", val),
    }
}

fn make_card(val: u8) -> Card {
    Card {
        pip: pip_for_value(val),
        suit: random_suit(),
    }
}

/// Build a (player_hand, dealer_hand) for a given TableIndex.
pub fn build_hand_for_index(index: &TableIndex) -> (Hand, Hand) {
    let row = index.row_index();
    let col = index.col_index().value(); // 1=Ace, 2-10

    let dealer_card = make_card(col);
    let mut dealer = Hand::default();
    dealer.add_card(dealer_card);

    let mut player = Hand::default();

    match index.table_type() {
        TableType::Hard => build_hard_hand(&mut player, row),
        TableType::Soft => build_soft_hand(&mut player, row),
        TableType::Split => build_split_hand(&mut player, row),
        TableType::Surrender => build_hard_hand(&mut player, row),
    }

    (player, dealer)
}

/// Build a hard hand totaling `total`.
/// Picks two non-ace cards that sum to `total`, avoiding pairs (to not trigger split).
fn build_hard_hand(hand: &mut Hand, total: u8) {
    let mut rng = thread_rng();

    // Valid first card range: 2..=10, second card = total - first, also 2..=10
    let min_first = total.saturating_sub(10).max(2);
    let max_first = (total - 2).min(10);

    if min_first > max_first {
        // Fallback for very low totals (e.g., total=4 only option is 2+2)
        let half = total / 2;
        hand.add_card(make_card(half));
        hand.add_card(make_card(total - half));
        return;
    }

    // Try to avoid pairs
    let mut first = rng.gen_range(min_first..=max_first);
    let mut second = total - first;

    // If it's a pair and we have room to adjust, shift by 1
    if first == second && max_first > min_first {
        if first < max_first {
            first += 1;
        } else {
            first -= 1;
        }
        second = total - first;
    }

    hand.add_card(make_card(first));
    hand.add_card(make_card(second));
}

/// Build a soft hand totaling `total` (e.g., soft 17 = Ace + 6).
fn build_soft_hand(hand: &mut Hand, total: u8) {
    hand.add_card(make_card(1)); // Ace (will count as 11)
    let other = total - 11;
    hand.add_card(make_card(other));
}

/// Build a split hand (pair) for the given row value.
/// Row 1 = Aces, Row 2-10 = that pip value.
fn build_split_hand(hand: &mut Hand, row: u8) {
    let val = if row == 1 { 11 } else { row }; // Ace has value 11 for card creation
    hand.add_card(make_card(val));
    hand.add_card(make_card(val));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strat::{new_table_index, ColIndex, RowIndex};

    fn make_index(tt: TableType, row: u8, col: u8) -> TableIndex {
        let ri = RowIndex::new(tt, row).unwrap();
        let ci: ColIndex = col.to_string().parse().unwrap();
        new_table_index(ri, ci)
    }

    #[test]
    fn test_hard_hand_total() {
        for total in 8..=17 {
            let idx = make_index(TableType::Hard, total, 5);
            let (player, dealer) = build_hand_for_index(&idx);
            assert_eq!(player.total(), total, "hard total mismatch for {}", total);
            assert!(
                !player.is_soft(),
                "hard hand should not be soft for {}",
                total
            );
            assert_eq!(player.num_cards(), 2);
            assert_eq!(dealer.num_cards(), 1);
        }
    }

    #[test]
    fn test_soft_hand_total() {
        for total in 13..=21 {
            let idx = make_index(TableType::Soft, total, 3);
            let (player, _) = build_hand_for_index(&idx);
            assert_eq!(player.total(), total, "soft total mismatch for {}", total);
            assert!(player.is_soft(), "soft hand should be soft for {}", total);
        }
    }

    #[test]
    fn test_split_hand() {
        for row in 1..=10 {
            let idx = make_index(TableType::Split, row, 7);
            let (player, _) = build_hand_for_index(&idx);
            assert!(
                player.splittable(),
                "split hand should be splittable for row {}",
                row
            );
            assert_eq!(player.num_cards(), 2);
        }
    }

    #[test]
    fn test_dealer_hand() {
        let idx = make_index(TableType::Hard, 12, 1); // dealer Ace
        let (_, dealer) = build_hand_for_index(&idx);
        assert_eq!(dealer.num_cards(), 1);
        // ColIndex 1 = Ace, card value = 11
        assert_eq!(dealer.first_card().unwrap().value(), 11);
    }
}
