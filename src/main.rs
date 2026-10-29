/*
README

IcoLang — An Emoji-Based Programming Language

Motivation:
- Programming languages are symbolic systems.
- Emojis are a universal symbolic vocabulary.
- This language explores whether computation can be expressed
  using emojis alone, without sacrificing rigor.

Design Principles:
- Emoji-only syntax
- Minimal instruction set
- Deterministic execution
- Fast compilation
- Single-file implementation

Computational Model:
- Unicode scalar memory tape (32-bit cells)
- Movable data pointer
- Sequential instruction execution
- Explicit control flow via jump instructions

Instruction Set Overview:
➕ Increment current cell (wraps at 2³²−1)
➖  Decrement current cell (wraps at 0)
➡️  Move pointer right
⬅️  Move pointer left
🔁  Loop start (jump if zero)
🔚  Loop end (jump if non-zero)
📤  Output current cell as Unicode character
🔢  Output current cell as emoji digits
📋  Copy current cell to clipboard
📥📋  Paste from clipboard to current cell
📤📋  Move from clipboard to current cell
📥  Input one byte into current cell
🟰 Compare current cell with right cell; store result (1 or 0) in cell two positions to the right
🔼 Compare current cell with right cell; store result (1 if greater, else 0) in cell two positions to the right
🔽 Compare current cell with right cell; store result (1 if less, else 0) in cell two positions to the right

Notes:
- Emojis are parsed as Unicode grapheme clusters.
- Loops are resolved at compile time.
- The VM executes a precompiled instruction stream.
*/

use unicode_segmentation::UnicodeSegmentation;
const MAX_MEMORY: usize = 1_000_000;
const MAX_STEPS: usize = 10_000_000;


#[derive(Debug, Clone, Copy)]
enum Token {
    Inc,
    Dec,
    Right,
    Left,
    LoopStart,
    LoopEnd,
    Input,
    Output,
    NumOutput,
    DebugToggle,
    Breakpoint,
    Number(u32),
    Clear,
    Equal,
    Greater,
    Less,
    CopyToClipboard,
    PasteFromClipboard,
    MoveFromClipboard,
}

#[derive(Debug, Clone, Copy)]
enum Instr {
    Inc,
    Dec,
    Right,
    Left,
    JumpIfZero(usize),
    JumpIfNonZero(usize),
    Input,
    Output,
    NumOutput,
    DebugToggle,
    Set(u32),
    Breakpoint,
    Clear,
    Equal,
    Greater,
    Less,
    CopyToClipboard,
    PasteFromClipboard,
    MoveFromClipboard,
}

#[derive(Debug)]
enum IcoError {
    UnknownEmoji(String),
    InvalidDigit(String),
    UnterminatedNumberLiteral,
    UnmatchedLoopStart,
    UnmatchedLoopEnd,
    PointerUnderflow,
    MemoryOverflow,
    StepLimitExceeded,
    NumberOverflow,
    IoError,
}

use std::fmt;

impl fmt::Display for IcoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IcoError::UnknownEmoji(s) => write!(f, "Unknown emoji: {}", s),
            IcoError::InvalidDigit(s) => write!(f, "Invalid digit emoji: {}", s),
            IcoError::UnmatchedLoopStart => write!(f, "Unmatched 🔁"),
            IcoError::UnmatchedLoopEnd => write!(f, "Unmatched 🔚"),
            IcoError::PointerUnderflow => write!(f, "Pointer underflow"),
            IcoError::MemoryOverflow => write!(f, "Memory overflow"),
            IcoError::IoError => write!(f, "I/O error"),
            IcoError::UnterminatedNumberLiteral => write!(f, "Unterminated 🔓 number literal"),
            IcoError::StepLimitExceeded => write!(f, "Instruction step limit exceeded"),
            IcoError::NumberOverflow => write!(f, "Number literal overflow"),
        }
    }
}

fn digit_from_emoji(g: &str) -> Option<u32> {
    match g {
        "0️⃣" => Some(0),
        "1️⃣" => Some(1),
        "2️⃣" => Some(2),
        "3️⃣" => Some(3),
        "4️⃣" => Some(4),
        "5️⃣" => Some(5),
        "6️⃣" => Some(6),
        "7️⃣" => Some(7),
        "8️⃣" => Some(8),
        "9️⃣" => Some(9),
        _ => None,
    }
}


fn lex(src: &str, strict: bool) -> Result<Vec<Token>, IcoError> {
    let mut tokens = Vec::new();
    let mut iter = src.graphemes(true).peekable();

    while let Some(g) = iter.next() {
        if g.trim().is_empty() {
            continue;
        }

        match g {
            "➡️" => tokens.push(Token::Right),
            "⬅️" => tokens.push(Token::Left),
            "🔁" => tokens.push(Token::LoopStart),
            "🔚" => tokens.push(Token::LoopEnd),
            "📥" => {
            if let Some(&next) = iter.peek() {
                if next == "📋" {
                    iter.next();
                    tokens.push(Token::PasteFromClipboard);
                } else {
                    tokens.push(Token::Input);
                }
            } else {
                tokens.push(Token::Input);
            }
            }

            "📤" => {
            if let Some(&next) = iter.peek() {
                if next == "📋" {
                    iter.next();
                    tokens.push(Token::MoveFromClipboard);
                } else {
                    tokens.push(Token::Output);
                }
            } else {
                tokens.push(Token::Output);
            }
            }

            "📋" => tokens.push(Token::CopyToClipboard),

            "🔢" => tokens.push(Token::NumOutput),
            "🐞" => tokens.push(Token::DebugToggle),
            "🧱" => tokens.push(Token::Breakpoint),
            "🧹" => tokens.push(Token::Clear),
            "🟰" => tokens.push(Token::Equal),
            "🔼" => tokens.push(Token::Greater),
            "🔽" => tokens.push(Token::Less),

            "➕" => tokens.push(Token::Inc),
            "➖" => tokens.push(Token::Dec),

            "💭" => {
                let mut closed = false;

                while let Some(next) = iter.next() {
                    if next == "💭" {
                        closed = true;
                        break;
                    }
                }

                if !closed {
                    return Err(IcoError::UnterminatedNumberLiteral);
                }
            }
            
            "🔓" => {
                let mut value: u32 = 0;
                let mut found_end = false;

                while let Some(next) = iter.next() {
                    if next == "🔒" {
                        found_end = true;
                        break;
                    }

                    if next.trim().is_empty() {
                        continue;
                    }

                    if let Some(d) = digit_from_emoji(next) {
                        value = value
                            .checked_mul(10)
                            .and_then(|v| v.checked_add(d))
                            .ok_or(IcoError::NumberOverflow)?;
                    } else {
                        return Err(IcoError::InvalidDigit(next.to_string()));
                    }
                }

                if !found_end {
                    return Err(IcoError::UnterminatedNumberLiteral);
                }

                tokens.push(Token::Number(value));
            }

            _ => {
                if strict {
                    return Err(IcoError::UnknownEmoji(g.to_string()));
                }
            }
        }
    }

    Ok(tokens)
}


fn compile(tokens: &[Token]) -> Result<Vec<Instr>, IcoError> {
    let mut instrs = Vec::new();
    let mut stack = Vec::new();

    for t in tokens {
        match t {
            Token::Inc => instrs.push(Instr::Inc),
            Token::Dec => instrs.push(Instr::Dec),
            Token::Right => instrs.push(Instr::Right),
            Token::Left => instrs.push(Instr::Left),
            Token::Input => instrs.push(Instr::Input),
            Token::Output => instrs.push(Instr::Output),
            Token::NumOutput => instrs.push(Instr::NumOutput),
            Token::DebugToggle => instrs.push(Instr::DebugToggle),
            Token::LoopStart => {
                stack.push(instrs.len());
                instrs.push(Instr::JumpIfZero(0));
            }
            Token::LoopEnd => {
                let start = stack.pop().ok_or(IcoError::UnmatchedLoopEnd)?;
                let end = instrs.len();
                instrs[start] = Instr::JumpIfZero(end + 1);
                instrs.push(Instr::JumpIfNonZero(start + 1));
            }
            Token::Breakpoint => instrs.push(Instr::Breakpoint),
            Token::Number(n) => instrs.push(Instr::Set(*n)),
            Token::Clear => instrs.push(Instr::Clear),
            Token::Equal => instrs.push(Instr::Equal),
            Token::Greater => instrs.push(Instr::Greater),
            Token::Less => instrs.push(Instr::Less),
            Token::CopyToClipboard => instrs.push(Instr::CopyToClipboard),
            Token::PasteFromClipboard => instrs.push(Instr::PasteFromClipboard),
            Token::MoveFromClipboard => instrs.push(Instr::MoveFromClipboard),
        }
    }

    if !stack.is_empty() {
        return Err(IcoError::UnmatchedLoopStart);
    }

    Ok(instrs)


}

fn print_number_emoji(mut n: u32) {
    if n == 0 {
        print!("0️⃣ ");
        return;
    }

    let mut digits = Vec::new();
    while n > 0 {
        digits.push(n % 10);
        n /= 10;
    }

    for d in digits.iter().rev() {
        match d {
            0 => print!("0️⃣ "),
            1 => print!("1️⃣ "),
            2 => print!("2️⃣ "),
            3 => print!("3️⃣ "),
            4 => print!("4️⃣ "),
            5 => print!("5️⃣ "),
            6 => print!("6️⃣ "),
            7 => print!("7️⃣ "),
            8 => print!("8️⃣ "),
            9 => print!("9️⃣ "),
            _ => {}
        }
    }
}


fn print_debug_state(ip: usize, ptr: usize, tape: &[u32], program: &[Instr]) {
    println!();
    println!("=== VM STATE ===");
    println!("ip: {}", ip);
    println!("ptr: {}", ptr);

    if let Some(instr) = program.get(ip) {
        println!("next: {:?}", instr);
    }

    println!("memory:");

    let window = 4;
    let start = ptr.saturating_sub(window);
    let end = (ptr + window + 1).min(tape.len());

    for i in start..end {
        if i == ptr {
            print!("[{:>15}] ", tape[i]);
        } else {
            print!("{:>18} ", tape[i]);
        }
    }

    println!();
    println!("================");
}

use std::{thread, time::Duration};

fn render_visual(
    ip: usize,
    ptr: usize,
    tape: &[u32],
    program: &[Instr],
    steps: usize,
    clipboard: &Option<u32>,
    
) {
    print!("\x1B[2J\x1B[1;1H");

    println!("=== IcoLang Live Visualizer ===");
    println!("Step: {}", steps);
    println!("IP: {}", ip);
    println!("PTR: {}", ptr);

    if let Some(instr) = program.get(ip) {
        println!("Current Instruction: {:?}", instr);
    }

    match clipboard {
        Some(v) => println!("Clipboard: {}", v),
        None => println!("Clipboard: empty"),
    }

    println!();
    println!("Tape:");

    let window = 5;
    let start = ptr.saturating_sub(window);
    let end = (ptr + window + 1).min(tape.len());

    for i in start..end {
        if i == ptr {
            print!("[{:>15}] ", tape[i]);
        } else {
            print!("{:>18} ", tape[i]);
        }
    }

    println!();
    println!("===============================");

    use std::io::{self, Write};
    io::stdout().flush().unwrap();

    thread::sleep(Duration::from_millis(75));
}

fn repl() {
    use std::io::{self, Write};
    let strict = false;
    let safe = false;
    println!("IcoLang REPL");
    println!("Type emoji code. Ctrl+C to exit.");

    loop {
        print!(">> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let tokens = match lex(&input, strict) {
            Ok(t) => t,
            Err(e) => {
                println!("Lex error: {}", e);
                continue;
            }
        };

        let program = match compile(&tokens) {
            Ok(p) => p,
            Err(e) => {
                println!("Compile error: {}", e);
                continue;
            }
        };

        match run(&program, false, strict, safe) {
            Ok(_) => {}
            Err(e) => println!("Runtime error: {}", e),
        }

        println!();
    }
}


fn run(program: &[Instr], visual: bool, strict: bool, safe: bool) -> Result<(usize, usize, usize), IcoError> {
    let mut tape = vec![0u32; 1];
    let mut ptr = 0usize;
    let mut ip = 0usize;
    let mut steps: usize = 0;
    let mut clipboard: Option<u32> = None;
    
    while ip < program.len() {
        steps += 1;
        if steps > MAX_STEPS {
            return Err(IcoError::StepLimitExceeded);
        }
        match program[ip] {
            Instr::Inc => tape[ptr] = tape[ptr].wrapping_add(1),
            Instr::Dec => tape[ptr] = tape[ptr].wrapping_sub(1),
            Instr::Clear => {
                tape[ptr] = 0;
            }
            Instr::Right => {
                ptr += 1;
                if ptr >= tape.len() {
                    if tape.len() >= MAX_MEMORY {
                        return Err(IcoError::MemoryOverflow);
                    }
                    tape.push(0);
                }
            }

            Instr::Left => {
            if ptr == 0 {
                if strict || safe {
                    return Err(IcoError::PointerUnderflow);
                }
            } else {
                ptr -= 1;
            }
    }

            Instr::Input => {
                use std::io::BufRead;
                let stdin = std::io::stdin();
                let mut line = String::new();
                stdin.lock().read_line(&mut line).map_err(|_| IcoError::IoError)?;
                let trimmed = line.trim();
                let value: u32 = trimmed.parse().map_err(|_| IcoError::IoError)?;
                tape[ptr] = value;
            }

            Instr::Output => {
                if let Some(c) = char::from_u32(tape[ptr]) {
                    print!("{}", c);
                }
            }

            Instr::NumOutput => print_number_emoji(tape[ptr]),

            Instr::DebugToggle => {
                if !safe {
                    print_debug_state(ip, ptr, &tape, &program);
                }
            }

            Instr::JumpIfZero(target) => {
                if tape[ptr] == 0 {
                    ip = target;
                    continue;
                }
            }

            Instr::JumpIfNonZero(target) => {
                if tape[ptr] != 0 {
                    ip = target;
                    continue;
                }
            }


            Instr::Breakpoint => {
                if !safe {
                println!("🧱 Breakpoint reached");
                println!("[ip={}] [ptr={}] [cell={}]", ip, ptr, tape[ptr]);
                use std::io::{self, Read};
                println!("Press Enter to continue");
                let mut _buf = [0u8; 1];
                let _ = io::stdin().read(&mut _buf);
                }
            }


            Instr::Set(n) => {
                tape[ptr] = n;
            }

            Instr::Equal => {
                let lhs = tape[ptr];
                let rhs = tape.get(ptr + 1).copied().unwrap_or(0);

                let result = if lhs == rhs { 1 } else { 0 };

                if ptr + 2 >= tape.len() {
                    let needed = ptr + 3;
                    if needed > MAX_MEMORY {
                    return Err(IcoError::MemoryOverflow);
                }
                tape.resize(needed, 0);
                }

                tape[ptr + 2] = result;
            }


            Instr::Greater => {
                let lhs = tape[ptr];
                let rhs = tape.get(ptr + 1).copied().unwrap_or(0);

                let result = if lhs > rhs { 1 } else { 0 };

                if ptr + 2 >= tape.len() {
                    let needed = ptr + 3;
                    if needed > MAX_MEMORY {
                        return Err(IcoError::MemoryOverflow);
                    }
                    tape.resize(needed, 0);
                }

                tape[ptr + 2] = result;
            }


            Instr::Less => {
                let lhs = tape[ptr];
                let rhs = tape.get(ptr + 1).copied().unwrap_or(0);

                let result = if lhs < rhs { 1 } else { 0 };

                if ptr + 2 >= tape.len() {
                    let needed = ptr + 3;
                if needed > MAX_MEMORY {
                    return Err(IcoError::MemoryOverflow);
                }
                tape.resize(needed, 0);
                }

                tape[ptr + 2] = result;
            }

            Instr::CopyToClipboard => {
                clipboard = Some(tape[ptr]);
            }

            Instr::PasteFromClipboard => {
                if let Some(value) = clipboard {
                    tape[ptr] = value;
            }
            }

            Instr::MoveFromClipboard => {
                if let Some(value) = clipboard {
                    tape[ptr] = value;
                    clipboard = None;
                }
            }
        }
        if visual {
            render_visual(ip, ptr, &tape, program, steps, &clipboard);
        }
        ip += 1;
    }


    Ok((steps, ptr, tape.len()))
    }


use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  icolang <file.icol>");
        eprintln!("  icolang run <file.icol>");
        process::exit(1);
    }

    let strict = args.iter().any(|a| a == "--strict");
    let safe = args.iter().any(|a| a == "--safe");
    let visual = args.iter().any(|a| a == "--visual");

// Filter out flag arguments when determining mode/filename
    let non_flag_args: Vec<&String> = args.iter()
        .filter(|arg| !arg.starts_with("--"))
        .collect();

    if args.len() == 2 && args[1] == "repl" {
        repl();
        return;
    }

    if non_flag_args.len() < 2 {
        eprintln!("IcoLang - Emoji-Based Programming Language");
        eprintln!();
        eprintln!("Usage:");
        eprintln!("  icolang <file.icol>              # Run program");
        eprintln!("  icolang repl                    # Interactive mode");
        eprintln!("  icolang run <file.icol>         # Run program explicitly");
        eprintln!("  icolang --safe <file.icol>      # Disable debugging features");
        eprintln!("  icolang --strict <file.icol>    # Error on unknown emojis");
        process::exit(1);
    }

    let (mode, filename) = if non_flag_args.len() == 2 {
        ("run", non_flag_args[1])
    } else {
        (&non_flag_args[1][..], non_flag_args[2])
    };


    if mode != "run" {
        eprintln!("Unknown command: {}", mode);
        eprintln!("Supported commands:");
        eprintln!("  run");
        process::exit(1);
    }

    if !filename.ends_with(".icol") {
        eprintln!("Error: File must have .icol extension");
        process::exit(1);
    }

    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Error: Could not read file {}", filename);
            process::exit(1);
        }
    };

    let tokens = match lex(&source, strict) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lex error: {}", e);
            process::exit(1);
        }
    };

    let program = match compile(&tokens) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Compile error: {}", e);
            process::exit(1);
        }
    };

    use std::time::Instant;
    let start = Instant::now();

    match run(&program, visual, strict, safe) {
        Ok((steps, ptr, mem)) => {
            let duration = start.elapsed();
            println!("\n--- Execution Stats ---");
            println!("Steps executed: {}", steps);
            println!("Final pointer position: {}", ptr);
            println!("Memory cells used: {}", mem);
            println!("Execution time: {:.2?}", duration);
        }
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            process::exit(1);
        }
    }
}

