# Terminal Casino

A terminal-based casino game featuring Baccarat with multiple game modes and bonus bets.

## Features

### Game Modes
- **Classic**: Traditional Baccarat with 5% banker commission
- **No Commission**: Banker wins pay 1:1, except banker 6 pays 1:2
- **Speed**: Simplified payouts with tie at 8:1
- **EZ Baccarat**: Includes Dragon 7 and Panda 8 special bets

### Bonus Bets
- Player Pair / Banker Pair (11:1)
- Either Pair (5:1)
- Perfect Pair (25:1)
- Player/Banker Dragon Bonus (up to 30:1)
- Lucky 6 (12:1 or 20:1)

### Statistics Tracking
- Win rates and round history
- Natural wins and pair hits
- Bonus bet performance

## Installation

```bash
cargo build --release
```

## Usage

```bash
cargo run
```

### Controls
- **[P]** Bet on Player
- **[B]** Bet on Banker
- **[T]** Bet on Tie
- **[M]** Change game mode
- **[1-5]** Set bet amount ($10-$1000)
- **[F1-F4]** Toggle bonus bets
- **[S]** Show/hide statistics
- **[SPACE]** Deal cards
- **[Q/ESC]** Quit

## Development

Built with Rust using:
- `crossterm` for terminal UI
- `rand` for card shuffling
- `bytemuck` for efficient data structures

## License

MIT