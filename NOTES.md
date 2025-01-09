Config Screen

Game:
Full
3-card

Questions:
Random
Targeted [Configure]


---------------------------------------------------------------------------

Ideas for a bj strategy trainer:

- Knows basic strategy
- Hands can be of two types:
    - **Random**: the cards are dealt fully randomized as if from a deck or shoe.
    - **Targeted**: A "cell" in the strategy table is chosen, and a hand is dealt that matches
        - Can choose to have a specific "type" of hand dealt:
            - either choose a table
            - or choose rows
- Each cell in the strategy table is tracked for:
    - # expressions
    - # correct (so, # wrong and percent correct)
    - # last 20 results
    - Leitner box #
    - next question date

Four tables:

- The X axis is always the dealers up card
- **Hard totals**
    - Y axis is player's hand total
- **Soft totals**
    - Y axis is player's hand total
- **Splits**
    - Y axis is the split card (hand total would work, but I think most peoplethink of this as the split card)
- **Surrender**
    - Y axis is player's hand total

Addressing the four tables:

- Rows:
    - Hard:##
        - when printed, ## will always be two digits and in the range [2, 21]
    - Soft:##
        - when printed, ## will always be two digits and in the range [2, 21]
    - Splits:##
        - when printed, ## will always be two digits and in the range [1, 10]
    - Surrender:##
        - when printed, ## will always be two digits and in the range [2, 20]
- Cols
    - after the Row, a comma and a dealer card. two digits. In the range [1, 10]
- Examples:
    - Hard:15,05 - Dealer: 5, Player: 10, 5
    - Soft:16,09 - Dealer: 9, Player: A, 5
    - Splits:01,10 - Dealer: 10, Player: A, A

- Game modes:
    - Play: full on blackjack. Deal cards. Then play the hand.
    - Strat: Just first three cards. Check response for basic strat. (This will be the first thing done.)
    - Targeted: either mode can be targeted and will just play the configured rows or tables.

- Configuration will be stored in .config (or the OS equivalent)
- Game state (Leitner stuff) will be stored in a game file.

