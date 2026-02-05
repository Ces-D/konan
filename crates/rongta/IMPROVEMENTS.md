# Rongta Module: Issues & Improvement Opportunities

This document catalogs identified issues in the `rongta` crate affecting speed, simplicity, and space complexity.

---

## Critical Bug

### Italic Handling Overwrites Underline State

**File:** `elements.rs:46-62`

```rust
impl ToPrintCommand for TextDecoration {
    fn to_print_command(&self, printer: &mut AnyPrinter) -> Result<()> {
        match self.bold {
            true => printer.bold(true)?,
            false => printer.bold(false)?,
        };
        match self.underline {
            true => printer.underline(UnderlineMode::Single)?,
            false => printer.underline(UnderlineMode::None)?,
        };
        match self.italic {
            true => printer.underline(UnderlineMode::Single)?,   // BUG: overwrites underline
            false => printer.underline(UnderlineMode::None)?,    // BUG: overwrites underline
        };
        Ok(())
    }
}
```

**Problem:** The italic branch calls `underline()` instead of an italic command. This means:

- If `underline=true, italic=false`: underline is set, then immediately turned off
- If `underline=false, italic=true`: underline is turned on (wrong behavior)

**Fix:** Either remove italic handling (ESC/POS printers typically don't support italic), or use a different command if the printer supports it.

---

## Speed Issues

### 1. Per-Character ESC/POS Command Overhead

**File:** `elements.rs:93-103`

**Severity:** High

```rust
impl ToPrintCommand for StyledChar {
    fn to_print_command(&self, printer: &mut AnyPrinter) -> Result<()> {
        let normalized_ch = cp437::normalize_char(self.ch).unwrap_or(self.ch);
        let ascii_content = cp437::cp437_char_only(normalized_ch)?;
        self.state.text_size.to_print_command(printer)?;       // ESC command
        self.state.text_decoration.to_print_command(printer)?; // 2-3 ESC commands
        printer.write(&ascii_content.to_string())?;            // write 1 char
        Ok(())
    }
}
```

**Problem:** Every character triggers 3-4 ESC/POS commands regardless of whether formatting changed. For a 48-character line with uniform formatting:

- Current: ~192 commands (4 per char)
- Optimal: ~5 commands (set format once, write 48 chars, reset)

**Impact:** ~40x more I/O operations than necessary.

**Fix:** Track current printer state and only emit formatting commands when state changes. Batch consecutive characters with identical formatting into single write calls.

```rust
// Proposed approach
struct PrintState {
    text_size: TextSize,
    text_decoration: TextDecoration,
}

fn print_line(line: &Line, printer: &mut AnyPrinter, state: &mut PrintState) -> Result<()> {
    let mut buffer = String::new();
    let mut current_format: Option<FormatState> = None;

    for styled_char in &line.chars {
        if current_format.as_ref() != Some(&styled_char.state) {
            // Flush buffer with previous format
            if !buffer.is_empty() {
                printer.write(&buffer)?;
                buffer.clear();
            }
            // Emit format change commands only if different from printer state
            if state.text_size != styled_char.state.text_size {
                styled_char.state.text_size.to_print_command(printer)?;
                state.text_size = styled_char.state.text_size;
            }
            // ... same for text_decoration
            current_format = Some(styled_char.state);
        }
        buffer.push(styled_char.ch);
    }
    if !buffer.is_empty() {
        printer.write(&buffer)?;
    }
    Ok(())
}
```

---

### 2. O(n²) Line Width Calculation

**File:** `rongta.rs:183-207`

**Severity:** Medium

```rust
fn add_char(&mut self, sch: StyledChar) -> Option<Line> {
    self.chars.push(sch);
    if self.visual_width() <= CPL as usize {  // O(n) - iterates all chars
        return None;
    }
    if let Some(wrap_point) = self.find_wrap_point() {  // O(n) - iterates again
        // ...
    }
}
```

**Problem:** Each `add_char()` call computes `visual_width()` by iterating all characters. For building a 48-character line:

- Iterations: 1 + 2 + 3 + ... + 48 = 1,176

**Impact:** Quadratic time complexity in line length.

**Fix:** Track cumulative width incrementally.

```rust
struct Line {
    pub chars: Vec<StyledChar>,
    pub justify_content: Justify,
    cached_width: usize,  // Add this
}

fn add_char(&mut self, sch: StyledChar) -> Option<Line> {
    let char_width = sch.state.text_size.char_width();
    self.cached_width += char_width;
    self.chars.push(sch);

    if self.cached_width <= CPL as usize {
        return None;
    }
    // ... wrap logic
}
```

---

### 3. Heap Allocation Per Character in Normalization

**File:** `cp437.rs:66-78`

**Severity:** Low-Medium

```rust
.flat_map(|ch| {
    if ch == '\u{2026}' {
        vec!['.', '.', '.']  // Heap allocation
    } else if let Some(replacement) = normalize_char(ch) {
        vec![replacement]    // Heap allocation
    } else {
        vec![ch]             // Heap allocation
    }
})
```

**Problem:** Creates a `Vec` on the heap for every character during normalization.

**Impact:** Unnecessary allocator pressure, cache misses.

**Fix:** Use stack-based iterators.

```rust
use std::iter::{once, repeat};

.flat_map(|ch| {
    if ch == '\u{2026}' {
        Either::Left(repeat('.').take(3))
    } else {
        Either::Right(once(normalize_char(ch).unwrap_or(ch)))
    }
})

// Or use a small array iterator:
.flat_map(|ch| {
    let (arr, len) = if ch == '\u{2026}' {
        (['.', '.', '.', '\0'], 3)
    } else {
        ([normalize_char(ch).unwrap_or(ch), '\0', '\0', '\0'], 1)
    };
    arr.into_iter().take(len)
})
```

---

## Space Complexity Issues

### 1. Per-Character Format State Storage

**File:** `elements.rs:88-92`

**Severity:** Medium

```rust
pub struct StyledChar {
    pub ch: char,            // 4 bytes
    pub state: FormatState,  // 4+ bytes
}

pub struct FormatState {
    pub text_size: TextSize,           // 1 byte (enum)
    pub text_decoration: TextDecoration, // 3 bytes (3 bools)
}
```

**Problem:** Each character stores its own formatting state copy. With padding, `StyledChar` is likely 8-12 bytes per character.

**Impact:** A 1000-character receipt uses 8-12 KB for character data alone.

**Alternative:** Run-length encoding or span-based representation.

```rust
// Span-based approach
struct TextSpan {
    text: String,
    format: FormatState,
}

struct Line {
    spans: Vec<TextSpan>,
    justify_content: Justify,
}
```

This stores formatting once per span rather than per character. A uniformly-formatted 48-char line uses ~56 bytes instead of ~384 bytes.

---

### 2. Full Document Buffering Before Print

**File:** `rongta.rs:313-353`

**Severity:** Low

```rust
pub fn print_to(&self, printer: &mut AnyPrinter, rows: Option<u32>) -> anyhow::Result<()> {
    // Iterates self.lines which holds the entire document
    for line in &self.lines {
        // ...
    }
}
```

**Problem:** `PrintBuilder` accumulates all lines in memory before printing.

**Impact:** Memory scales with document size. For very long receipts, this could be problematic on embedded systems.

**Alternative:** Streaming architecture that flushes lines as they're completed.

---

## Simplicity Issues

### 1. RESOLVED -- AnyPrinter Match Boilerplate

**File:** `rongta.rs:26-133`

**Severity:** Medium (maintainability)

```rust
impl AnyPrinter {
    pub fn feed(&mut self) -> Result<()> {
        match self {
            AnyPrinter::Usb(p) => { p.feed()?; }
            AnyPrinter::Network(p) => { p.feed()?; }
        }
        Ok(())
    }
    // ... 8 more methods with identical pattern
}
```

**Problem:** 9 methods with identical match structure. Adding a new driver requires updating all 9 methods.

**Impact:** ~100 lines of repetitive code.

**Fix Option A:** Macro to generate methods.

```rust
macro_rules! delegate_printer_method {
    ($method:ident $(, $arg:ident : $ty:ty)*) => {
        pub fn $method(&mut self $(, $arg: $ty)*) -> Result<()> {
            match self {
                AnyPrinter::Usb(p) => p.$method($($arg),*)?,
                AnyPrinter::Network(p) => p.$method($($arg),*)?,
            }
            Ok(())
        }
    };
}

impl AnyPrinter {
    delegate_printer_method!(feed);
    delegate_printer_method!(print);
    delegate_printer_method!(print_cut);
    delegate_printer_method!(write, text: &str);
    delegate_printer_method!(bold, enabled: bool);
    // ...
}
```

**Fix Option B:** Internal helper method.

```rust
impl AnyPrinter {
    fn with_printer<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut dyn PrinterOps) -> Result<R>,
    {
        match self {
            AnyPrinter::Usb(p) => f(p),
            AnyPrinter::Network(p) => f(p),
        }
    }
}
```

---

### 2. RESOLVED -- Unnecessary Pop/Push Pattern

**File:** `rongta.rs:251-274`

```rust
pub fn add_content(&mut self, content: &str) -> Result<()> {
    let mut current_line = self.lines.pop().unwrap_or_else(|| Line { ... });
    // ... modify current_line ...
    self.lines.push(current_line);
    Ok(())
}
```

**Problem:** Pops the last line, modifies it, pushes it back. This is semantically unclear and has edge case risks.

**Fix:** Use `last_mut()` with entry-like pattern.

```rust
pub fn add_content(&mut self, content: &str) -> Result<()> {
    if self.lines.is_empty() {
        self.lines.push(Line {
            justify_content: self.current_line_justify_content(),
            ..Default::default()
        });
    }
    let current_line = self.lines.last_mut().unwrap();
    // ... modify current_line directly ...
    Ok(())
}
```

---

## Summary Table

| Category   | Issue                       | Location           | Severity | Impact                 |
| ---------- | --------------------------- | ------------------ | -------- | ---------------------- |
| Bug        | Italic overwrites underline | elements.rs:56-59  | Critical | Incorrect output       |
| Speed      | Per-char ESC/POS commands   | elements.rs:93-103 | High     | ~40x excess I/O        |
| Speed      | O(n²) width calculation     | rongta.rs:183-207  | Medium   | Quadratic time         |
| Speed      | Vec alloc in normalization  | cp437.rs:66-78     | Low      | Allocator pressure     |
| Space      | Per-char format storage     | elements.rs:88-92  | Medium   | ~8x memory overhead    |
| Space      | Full document buffering     | rongta.rs:313-353  | Low      | Memory scales with doc |
| Simplicity | AnyPrinter boilerplate      | rongta.rs:26-133   | Medium   | 100+ redundant lines   |
| Simplicity | Pop/push line pattern       | rongta.rs:251-274  | Low      | Unclear semantics      |

---

## Recommended Priority

1. **Fix italic bug** - Correctness issue
2. **Batch ESC/POS commands** - Largest performance win
3. **Incremental width tracking** - Simple fix, removes O(n²)
4. **Macro for AnyPrinter** - Maintainability improvement
5. **Span-based text storage** - Larger refactor, consider if memory is constrained
