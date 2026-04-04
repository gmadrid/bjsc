# BJSC - Blackjack Strategy Card Trainer

An interactive trainer for memorizing blackjack basic strategy decisions.

## Project Structure

- `bjsc` (root) - Shared game logic library
- `bjsc-tui` - Terminal UI (ratatui + crossterm)
- `bjsc-web` - Web UI (Leptos CSR)

## Running the TUI

```
cargo run -p bjsc-tui
```

**Keys:** `h` hit | `s` stand | `d` double | `p` split | `q` quit

## Running the Web Version

### Prerequisites

```
rustup target add wasm32-unknown-unknown
cargo install trunk
```

### Start the dev server

```
cd bjsc-web
trunk serve
```

Opens at `http://127.0.0.1:8080`. Use the same keyboard shortcuts or click the buttons.

### Build for production

```
cd bjsc-web
trunk build --release
```

Output goes to `bjsc-web/dist/`.
