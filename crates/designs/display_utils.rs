//! Shared display utilities for adapters that render content to PrintBuilder.
//!
//! These functions ensure consistent styling across different input formats
//! (Markdown, Tiptap JSON, etc.) when rendering to the thermal printer.

use anyhow::Result;
use rongta::{
    CPL, PrintBuilder,
    elements::{Justify, TextDecoration, TextSize},
};

/// Render strikethrough text by wrapping with dashes.
/// The printer doesn't support native strikethrough, so we use `--text--` format.
pub fn render_strikethrough(builder: &mut PrintBuilder, text: &str) -> Result<()> {
    builder.add_content("--")?;
    builder.add_content(text)?;
    builder.add_content("--")
}

/// Render a horizontal rule as a line of dashes.
pub fn render_horizontal_rule(builder: &mut PrintBuilder) -> Result<()> {
    builder.new_line();
    let dashes = "─".repeat(CPL as usize);
    builder.add_content(&dashes)?;
    builder.new_line();
    Ok(())
}

/// Get the text size and decoration for a heading level.
/// Returns (TextSize, TextDecoration) tuple.
///
/// Mapping:
/// - H1: ExtraLarge, no decoration
/// - H2: Large, bold
/// - H3: Large, no decoration
/// - H4+: Medium, bold
pub fn heading_style(level: u8) -> (TextSize, TextDecoration) {
    match level {
        1 => (TextSize::ExtraLarge, TextDecoration::default()),
        2 => (
            TextSize::Large,
            TextDecoration {
                bold: true,
                ..Default::default()
            },
        ),
        3 => (TextSize::Large, TextDecoration::default()),
        _ => (
            TextSize::Medium,
            TextDecoration {
                bold: true,
                ..Default::default()
            },
        ),
    }
}

/// Render a heading with the appropriate size and decoration.
/// Centers the content and resets styles after.
pub fn render_heading<F>(builder: &mut PrintBuilder, level: u8, render_content: F) -> Result<()>
where
    F: FnOnce(&mut PrintBuilder) -> Result<()>,
{
    let (size, decoration) = heading_style(level);
    builder.new_line();
    builder.set_text_size(size);
    builder.set_text_decoration(decoration);
    builder.set_justify_content(Justify::Center);
    render_content(builder)?;
    builder.new_line();
    builder.reset_styles();
    Ok(())
}

/// Render a blockquote with bold, underline, and centered text.
pub fn render_blockquote<F>(builder: &mut PrintBuilder, render_content: F) -> Result<()>
where
    F: FnOnce(&mut PrintBuilder) -> Result<()>,
{
    builder.new_line();
    builder.new_line();
    builder.set_text_decoration(TextDecoration {
        bold: true,
        italic: false,
        underline: true,
    });
    builder.set_justify_content(Justify::Center);
    render_content(builder)?;
    builder.new_line();
    builder.new_line();
    builder.reset_styles();
    Ok(())
}

/// Render a code block with bold text.
pub fn render_code_block(builder: &mut PrintBuilder, content: &str) -> Result<()> {
    builder.new_line();
    builder.new_line();
    builder.set_text_decoration(TextDecoration {
        bold: true,
        ..Default::default()
    });
    builder.add_content(content)?;
    builder.new_line();
    builder.new_line();
    builder.reset_styles();
    Ok(())
}

/// Render inline code with bold and underline.
pub fn render_inline_code(builder: &mut PrintBuilder, content: &str) -> Result<()> {
    builder.set_text_decoration(TextDecoration {
        bold: true,
        underline: true,
        ..Default::default()
    });
    builder.add_content(content)?;
    builder.reset_styles();
    Ok(())
}

/// Render a task item checkbox prefix.
/// Returns "[x] " for checked items and "[ ] " for unchecked.
pub fn task_item_prefix(checked: bool) -> &'static str {
    if checked { "[x] " } else { "[ ] " }
}

/// Render bold text.
pub fn render_bold<F>(builder: &mut PrintBuilder, render_content: F) -> Result<()>
where
    F: FnOnce(&mut PrintBuilder) -> Result<()>,
{
    builder.set_text_decoration(TextDecoration {
        bold: true,
        ..Default::default()
    });
    render_content(builder)?;
    builder.reset_styles();
    Ok(())
}

/// Render italic/emphasized text (rendered as underline since printer doesn't support italic).
pub fn render_italic<F>(builder: &mut PrintBuilder, render_content: F) -> Result<()>
where
    F: FnOnce(&mut PrintBuilder) -> Result<()>,
{
    builder.set_text_decoration(TextDecoration {
        underline: true,
        ..Default::default()
    });
    render_content(builder)?;
    builder.reset_styles();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_style_h1() {
        let (size, decoration) = heading_style(1);
        assert_eq!(size, TextSize::ExtraLarge);
        assert!(!decoration.bold);
    }

    #[test]
    fn test_heading_style_h2() {
        let (size, decoration) = heading_style(2);
        assert_eq!(size, TextSize::Large);
        assert!(decoration.bold);
    }

    #[test]
    fn test_heading_style_h3() {
        let (size, decoration) = heading_style(3);
        assert_eq!(size, TextSize::Large);
        assert!(!decoration.bold);
    }

    #[test]
    fn test_heading_style_h4_and_beyond() {
        for level in 4..=6 {
            let (size, decoration) = heading_style(level);
            assert_eq!(size, TextSize::Medium);
            assert!(decoration.bold);
        }
    }

    #[test]
    fn test_task_item_prefix_checked() {
        assert_eq!(task_item_prefix(true), "[x] ");
    }

    #[test]
    fn test_task_item_prefix_unchecked() {
        assert_eq!(task_item_prefix(false), "[ ] ");
    }
}
