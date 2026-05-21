# 🟦 Tetris-RS

A fully-featured terminal Tetris game written in Rust. Runs entirely in your terminal — no GUI, no browser, just pure Rust.





***

## ✨ Features

- 🌍 **Language selection** at startup — English or German
- 🎮 **All 7 official Tetrominoes** (I, O, T, S, Z, J, L)
- 🔄 **Full SRS rotation system** with official wall-kick tables
- 👻 **Ghost piece** showing where the block will land
- 🔒 **Hold piece** — swap the current piece for later
- ⚡ **Hard drop & soft drop** with bonus scoring
- 📈 **Progressive difficulty** — speed increases every 10 lines
- 🔢 **Combo scoring** — chain line clears for bonus points
- 🎨 **Colored pieces** with smooth ~60fps rendering
- ✅ **No flickering** — uses buffered differential rendering

***

## 📋 Requirements

- **Rust 1.70 or newer** — [install via rustup](https://rustup.rs/)
- A terminal that supports ANSI color codes
  - Windows: Windows Terminal or PowerShell 7+ (recommended)
  - macOS: Terminal.app or iTerm2
  - Linux: Any modern terminal emulator
- Terminal size: **minimum 52 columns × 24 rows** (bigger is better)

***

## 🚀 Installation & Running

### Option 1 — Clone and run directly

```bash
git clone https://github.com/Fantasy102/tetris-rs.git
cd tetris-rs
cargo run --release
```

### Option 2 — Install globally (play from anywhere)

```bash
cargo install --path .
tetris
```

### Option 3 — Build a standalone binary

```bash
cargo build --release
# Binary is at: target/release/tetris  (or tetris.exe on Windows)
./target/release/tetris
```

***

## 🕹️ Controls

| Key | Action |
|-----|--------|
| `A` / `←` | Move left |
| `D` / `→` | Move right |
| `W` / `↑` | Rotate clockwise |
| `S` / `↓` | Soft drop (faster fall) |
| `Space` | Hard drop (instant drop) |
| `C` | Hold current piece |
| `P` | Pause / Resume |
| `Q` / `Esc` | Quit |

> **Tip:** Hard drop scores 2 points per row the piece travels. Chaining line clears builds a combo multiplier!

***

## 📦 Project Structure

```
tetris-rs/
├── Cargo.toml          # Project manifest & dependencies
└── src/
    ├── main.rs         # Game loop, timing, entry point
    ├── game.rs         # Game logic: pieces, board, scoring, rotation
    ├── input.rs        # Keyboard input handling + language selection screen
    └── render.rs       # Terminal rendering engine
```

### Module overview

| File | Responsibility |
|------|---------------|
| `main.rs` | Main loop at ~60fps, gravity timer, Hard Drop timer reset |
| `game.rs` | All game state: board, pieces, SRS wall kicks, line clearing, scoring |
| `input.rs` | Non-blocking input polling, Release-event filtering (fixes Windows double-input) |
| `render.rs` | Buffered differential rendering to prevent flicker, UI panels |

***

## 🧮 Scoring System

| Action | Points |
|--------|--------|
| Soft drop (per row) | 1 × level |
| Hard drop (per row) | 2 × level |
| Single line clear | 100 × level |
| Double line clear | 300 × level |
| Triple line clear | 500 × level |
| Tetris (4 lines) | 800 × level |
| Combo bonus (per chain) | +50 × (combo − 1) × level |

Level increases by 1 every 10 lines cleared. Drop speed scales with level.

***

## 🔧 Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| [`crossterm`](https://crates.io/crates/crossterm) | 0.28 | Cross-platform terminal control (input, cursor, colors) |
| [`rand`](https://crates.io/crates/rand) | 0.8 | 7-bag randomizer for piece generation |

***

## 🪟 Windows Notes

Windows sends both `KeyPress` and `KeyRelease` events for every keypress. Without filtering, this causes each key to register twice. `input.rs` explicitly discards `KeyEventKind::Release` events, fixing this.

**Recommended terminal on Windows:** [Windows Terminal](https://aka.ms/terminal) for best color support and rendering.

***

## 🐛 Troubleshooting

**Terminal looks garbled / boxes instead of block characters**
> Your terminal font may not support Unicode block characters (`██`, `░░`). Use a font like [Cascadia Code](https://github.com/Microslop/cascadia-code), [JetBrains Mono](https://www.jetbrains.com/lp/mono/), or [Fira Code](https://github.com/tonsky/FiraCode).

**Compilation error: `error[E0554]: #![feature]` or similar**
> Make sure your Rust toolchain is up to date: `rustup update`

**Game is too fast / too slow**
> This is expected — speed scales with level. Level 1 is the slowest, reaching minimum drop speed (~50ms) around level 15.

**Colors not showing on Windows CMD**
> Use Windows Terminal or PowerShell 7+ instead of the legacy Command Prompt.

***

## 📄 License

MIT — do whatever you want with it.

***

*Built with 🦀 Rust by Fantasy1002*
