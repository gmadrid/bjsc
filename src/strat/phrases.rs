use crate::card::Pip::{Ace, Eight, Five, Four, Nine, Seven, Six, Ten, Three, Two};
use crate::rowidx;
use crate::strat::tableindex::RowIndex;
use crate::strat::tableindex::TableType::{Hard, Soft, Split, Surrender};
use std::collections::HashMap;
use std::sync::LazyLock;

macro_rules! phrase_row {
    ($m:expr, $t:expr, $c:expr, $p:expr) => {
        $m.insert(rowidx!($t, $c), $p);
    };
}

// These phrases are from the Blackjack Apprentice site.
// They are available at this publicly readable link: https://www.blackjackapprenticeship.com/blackjack-strategy-charts/
static PHRASES: LazyLock<HashMap<RowIndex, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    phrase_row!(
        m,
        Surrender,
        15,
        "15 surrenders against dealer 10, otherwise don’t surrender (revert to hard totals)."
    );
    phrase_row!(m, Surrender, 16, "16 surrenders against dealer 9 through Ace, otherwise don’t surrender (revert to hard totals).");

    phrase_row!(m, Split, Ace.value() - 10, "Always split Aces.");
    phrase_row!(
        m,
        Split,
        Two.value(),
        "A pair of 2’s splits against dealer 2 through 7, otherwise hit."
    );
    phrase_row!(
        m,
        Split,
        Three.value(),
        "A pair of 3’s splits against dealer 2 through 7, otherwise hit."
    );
    phrase_row!(
        m,
        Split,
        Four.value(),
        "A pair of 4’s splits against dealer 5 and 6 , otherwise hit."
    );
    phrase_row!(
        m,
        Split,
        Five.value(),
        "A pair of 5’s doubles against dealer 2 through 9 otherwise hit."
    );
    phrase_row!(
        m,
        Split,
        Six.value(),
        "A pair of 6’s splits against dealer 2 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Split,
        Seven.value(),
        "A pair of 7’s splits against dealer 2 through 7, otherwise hit."
    );
    phrase_row!(m, Split, Eight.value(), "Always split 8’s.");
    phrase_row!(
        m,
        Split,
        Nine.value(),
        "A pair of 9’s splits against dealer 2 through 9, except for 7, otherwise stand."
    );
    phrase_row!(m, Split, Ten.value(), "Never split tens.");

    phrase_row!(
        m,
        Soft,
        13,
        "Soft 13 (A,2) doubles against dealer 5 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Soft,
        14,
        "Soft 14 (A,3) doubles against dealer 5 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Soft,
        15,
        "Soft 15 (A,4) doubles against dealer 4 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Soft,
        16,
        "Soft 16 (A,5) doubles against dealer 4 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Soft,
        17,
        "Soft 17 (A,6) doubles against dealer 3 through 6, otherwise hit."
    );
    phrase_row!(m, Soft, 18, "Soft 18 (A,7) doubles against dealer 2 through 6, and hits against 9 through Ace, otherwise stand.");
    phrase_row!(
        m,
        Soft,
        19,
        "Soft 19 (A,8) doubles against dealer 6, otherwise stand."
    );
    phrase_row!(m, Soft, 20, "Soft 20 (A,9) always stands");
    phrase_row!(m, Soft, 21, "Is a Blackjack. Stand, idiot!");

    phrase_row!(m, Hard, 8, "8 always hits.");
    phrase_row!(
        m,
        Hard,
        9,
        "9 doubles against dealer 3 through 6 otherwise hit."
    );
    phrase_row!(
        m,
        Hard,
        10,
        "10 doubles against dealer 2 through 9 otherwise hit."
    );
    phrase_row!(m, Hard, 11, "11 always doubles.");
    phrase_row!(
        m,
        Hard,
        12,
        "12 stands against dealer 4 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Hard,
        13,
        "13 stands against dealer 2 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Hard,
        14,
        "14 stands against dealer 2 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Hard,
        15,
        "15 stands against dealer 2 through 6, otherwise hit."
    );
    phrase_row!(
        m,
        Hard,
        16,
        "16 stands against dealer 2 through 6, otherwise hit."
    );
    phrase_row!(m, Hard, 17, "17 and up always stands.");

    m
});

pub fn phrase_for_row(ri: RowIndex) -> &'static str {
    PHRASES
        .get(&ri)
        .unwrap_or(&"Internal Error: RowIndex unrecognized.")
}
