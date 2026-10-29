# 🎨 IcoLang

> **An Emoji-Based Programming Language implemented in Rust**

---

## 💡 Motivation
- Programming languages are symbolic systems.
- Emojis are a universal symbolic vocabulary.
- **IcoLang** explores whether computation can be expressed using emojis alone, without sacrificing rigor.

---

## ⚙️ Design Principles
- **Emoji-only syntax**: Programs are authored with expressive emojis.
- **Minimal instruction set**: Turing-complete, brainfuck-inspired tape architecture.
- **Deterministic execution**: Explicit control flow and jump resolution.
- **Fast compilation**: Tokenized with Unicode grapheme clustering and compiled into bytecode instructions.
- **Single-file implementation**: Clean, zero-bloat Rust engine.

---

## 🧠 Computational Model
- **Memory**: Unicode scalar memory tape (32-bit cells up to 1,000,000 cells).
- **Pointer**: Movable data pointer across the tape.
- **Execution**: Sequential instruction execution with jump tables for loops.
- **Step Limit**: Protected against infinite execution with configurable limits.

---

## 📋 Instruction Set

| Emoji | Instruction | Description |
| :---: | :--- | :--- |
| ➕ | Increment | Increment current cell (wraps at $2^{32}-1$) |
| ➖ | Decrement | Decrement current cell (wraps at 0) |
| ➡️ | Move Right | Move data pointer one cell right |
| ⬅️ | Move Left | Move data pointer one cell left |
| 🔁 | Loop Start | Jump past matching 🔚 if current cell is 0 |
| 🔚 | Loop End | Jump back to matching 🔁 if current cell is non-zero |
| 📤 | Output Char | Output current cell value as a Unicode character |
| 🔢 | Output Number | Output current cell value as emoji digits |
| 📥 | Input | Read one byte into current cell |
| 📋 | Copy | Copy current cell value to clipboard |
| 📥📋 | Paste | Paste clipboard value into current cell |
| 📤📋 | Move | Move clipboard value to current cell |
| 🟰 | Equal | Compare current cell with right cell; store result (1/0) two positions right |
| 🔼 | Greater Than | Compare current cell with right cell; store result (1 if greater, else 0) two positions right |
| 🔽 | Less Than | Compare current cell with right cell; store result (1 if less, else 0) two positions right |
| 🧹 | Clear | Reset current cell to 0 |

---

## 🚀 Getting Started

### Prerequisites
- [Rust & Cargo](https://www.rust-lang.org/tools/install) installed.

### Installation & Build
Clone the repository and build:
```bash
git clone https://github.com/theviolinplayer/icolang.git
cd icolang
cargo build --release
```

### Usage
```bash
# Run a program
cargo run -- <file.icol>

# Interactive REPL mode
cargo run -- repl

# Visual execution mode
cargo run -- --visual <file.icol>

# Safe mode (disables debugging features)
cargo run -- --safe <file.icol>

# Strict mode (fails on unknown emojis)
cargo run -- --strict <file.icol>
```

---

## 📂 Example Programs

Check out the sample `.icol` programs in the repository:
- `factorial.icol` — Factorial computation
- `add.icol` — Basic arithmetic addition
- `abc.icol` — String output
- `visual.icol` — Tape visualizer demo
- `debug.icol` — Debugging features demo

---

## 📄 License
This project is licensed under the [MIT License](LICENSE).
