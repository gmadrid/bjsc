# BJSC - Blackjack Strategy Card Trainer

An interactive trainer for memorizing blackjack basic strategy decisions.

## Project Structure

- `bjsc` (root) - Shared game logic library
- `bjsc-tui` - Terminal UI (ratatui + crossterm)
- `bjsc-web` - Web UI (Leptos CSR + Tailwind)
- `spaced-rep` - Generic spaced repetition engine (Leitner boxes)

## Running the TUI

```
cargo run -p bjsc-tui
```

On first run, opens your browser for Google sign-in. Auth is saved to `~/.bjsc/auth.json`.
Game progress syncs to Supabase and is also saved locally to `~/.bjsc/state.toml`.

**Keys:** `h` hit | `s` stand | `d` double | `p` split | `m` mode | `Tab` stats | `q` quit

## Running the Web Version

### Prerequisites

```
rustup target add wasm32-unknown-unknown
cargo install trunk
cd bjsc-web && pnpm install
```

### Start the dev server

```
cd bjsc-web
trunk serve
```

Opens at `http://127.0.0.1:8080`. Sign in with Google to sync progress via Supabase.

**Keys:** `h` hit | `s` stand | `d` double | `p` split | `m` mode | `Tab` stats

### Build for production

```
cd bjsc-web
trunk build --release
```

Output goes to `bjsc-web/dist/`.
