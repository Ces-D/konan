use anyhow::Result;
use rongta::{
    RongtaPrinter, ToBuilderCommand,
    elements::{FormatState, Justify, TextDecoration, TextSize},
};
use tiptap::OrderedListType;

/// Style the ListItem ::before pseudoelement
pub struct ListItemBefore {
    content: String,
    format: FormatState,
}
impl ListItemBefore {
    pub fn new_ordered(start: Option<u64>, ordinal: Option<OrderedListType>) -> Self {
        let start = start.unwrap_or(1);
        let value = match ordinal.unwrap_or_default() {
            OrderedListType::LowerCaseLetter => Self::letter_for_index(start - 1, false),
            OrderedListType::UpperCaseLetter => Self::letter_for_index(start - 1, true),
            OrderedListType::LowerCaseRoman => Self::roman_numeral(start, false),
            OrderedListType::UpperCaseRoman => Self::roman_numeral(start, true),
            OrderedListType::Number => start.to_string(),
        };
        Self {
            content: format!("{}. ", value),
            format: FormatState {
                text_size: TextSize::Medium,
                text_decoration: TextDecoration {
                    bold: true,
                    ..Default::default()
                },
            },
        }
    }
    pub fn new_unordered() -> Self {
        Self {
            content: "∙ ".to_string(),
            format: FormatState {
                text_size: TextSize::Medium,
                text_decoration: TextDecoration {
                    bold: true,
                    ..Default::default()
                },
            },
        }
    }
    /// Returns the alphabetic label for a 1-based index.
    /// Examples: 1 -> "a"/"A", 26 -> "z"/"Z", 27 -> "aa"/"AA".
    fn letter_for_index(index: u64, uppercase: bool) -> String {
        if index == 0 {
            return String::new();
        }
        let mut n = index;
        let mut s = String::new();
        while n > 0 {
            let rem = ((n - 1) % 26) as u8;
            let base = if uppercase { b'A' } else { b'a' };
            s.insert(0, (base + rem) as char);
            n = (n - 1) / 26;
        }
        s
    }
    /// Returns the Roman numeral for a positive integer (1..=3999).
    /// Set `uppercase` to control casing (e.g., 4 -> "iv" or "IV").
    fn roman_numeral(value: u64, uppercase: bool) -> String {
        if value == 0 || value > 3999 {
            return String::new();
        }
        let mut n = value;
        let vals: [u64; 13] = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
        let syms: [&str; 13] = [
            "M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I",
        ];
        let mut out = String::new();
        for (i, &v) in vals.iter().enumerate() {
            while n >= v {
                out.push_str(syms[i]);
                n -= v;
            }
        }
        if uppercase { out } else { out.to_lowercase() }
    }
}
impl ToBuilderCommand for ListItemBefore {
    fn to_builder_command(&self, builder: &mut RongtaPrinter) -> Result<()> {
        log::trace!("Justification ignored for list items");
        builder.new_line();
        builder.reset_styles();
        builder.set_justify_content(Justify::Left);
        builder.set_text_size(self.format.text_size);
        builder.set_text_decoration(self.format.text_decoration);
        builder.add_content(&self.content)
    }
}

pub struct TaskListBefore {
    content: String,
    format: FormatState,
}
impl TaskListBefore {
    pub fn new(checked: bool) -> Self {
        let content = if checked {
            "[■] ".to_string()
        } else {
            "[ ] ".to_string()
        };
        Self {
            content,
            format: FormatState {
                text_size: TextSize::Medium,
                text_decoration: TextDecoration {
                    bold: true,
                    ..Default::default()
                },
            },
        }
    }
}
impl ToBuilderCommand for TaskListBefore {
    fn to_builder_command(&self, builder: &mut RongtaPrinter) -> Result<()> {
        builder.new_line();
        builder.reset_styles();
        builder.set_text_size(self.format.text_size);
        builder.set_text_decoration(self.format.text_decoration);
        builder.add_content(&self.content)
    }
}
/// Renders all non-heading text
pub struct Text {
    content: String,
    format: FormatState,
}
impl Text {
    pub fn new(text: String, text_size: Option<TextSize>, bold: Option<bool>) -> Self {
        Self {
            content: text,
            format: FormatState {
                text_size: text_size.unwrap_or_default(),
                text_decoration: TextDecoration {
                    bold: bold.unwrap_or_default(),
                    ..Default::default()
                },
            },
        }
    }
}
impl ToBuilderCommand for Text {
    fn to_builder_command(&self, builder: &mut RongtaPrinter) -> Result<()> {
        builder.set_text_size(self.format.text_size);
        builder.set_text_decoration(self.format.text_decoration);
        builder.add_content(&self.content)
    }
}

pub struct Heading {
    content: String,
    format: FormatState,
}
impl Heading {
    pub fn new(text: String, level: Option<u8>) -> Self {
        let (text_size, text_decoration) = Self::heading_style(level.unwrap_or(3));
        Self {
            content: text.trim().to_string(),
            format: FormatState {
                text_size,
                text_decoration,
            },
        }
    }
    fn heading_style(level: u8) -> (TextSize, TextDecoration) {
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
}
impl ToBuilderCommand for Heading {
    fn to_builder_command(&self, builder: &mut RongtaPrinter) -> Result<()> {
        builder.new_line();
        builder.reset_styles();
        builder.set_text_size(self.format.text_size);
        builder.set_text_decoration(self.format.text_decoration);
        builder.set_justify_content(Justify::Center);
        builder.add_content(&self.content)?;
        builder.new_line();
        Ok(())
    }
}

pub struct BlockQuote {
    content: String,
    format: FormatState,
}
impl BlockQuote {
    pub fn new(text: String) -> Self {
        Self {
            content: text,
            format: FormatState {
                text_size: TextSize::Medium,
                text_decoration: TextDecoration {
                    bold: true,
                    ..Default::default()
                },
            },
        }
    }
}
impl ToBuilderCommand for BlockQuote {
    fn to_builder_command(&self, builder: &mut RongtaPrinter) -> Result<()> {
        builder.new_line();
        builder.reset_styles();
        builder.set_text_size(self.format.text_size);
        builder.set_text_decoration(self.format.text_decoration);
        builder.set_justify_content(Justify::Center);
        builder.add_content(&self.content)?;
        builder.new_line();
        Ok(())
    }
}

pub struct CodeBlock {
    content: String,
    format: FormatState,
}
impl CodeBlock {
    pub fn new(text: String) -> Self {
        Self {
            content: text,
            format: FormatState {
                text_size: TextSize::Medium,
                text_decoration: TextDecoration {
                    bold: true,
                    ..Default::default()
                },
            },
        }
    }
}
impl ToBuilderCommand for CodeBlock {
    fn to_builder_command(&self, builder: &mut RongtaPrinter) -> Result<()> {
        builder.new_line();
        builder.reset_styles();
        builder.set_text_size(self.format.text_size);
        builder.set_text_decoration(self.format.text_decoration);
        builder.set_justify_content(Justify::Left);
        builder.add_content(&self.content)?;
        builder.new_line();
        Ok(())
    }
}

pub struct HorizontalRule {
    content: String,
    format: FormatState,
}
impl HorizontalRule {
    pub fn new() -> Self {
        Self {
            content: "-".repeat(12),
            format: FormatState {
                text_size: TextSize::Large,
                text_decoration: TextDecoration {
                    bold: true,
                    ..Default::default()
                },
            },
        }
    }
}
impl ToBuilderCommand for HorizontalRule {
    fn to_builder_command(&self, builder: &mut RongtaPrinter) -> Result<()> {
        builder.new_line();
        builder.reset_styles();
        builder.set_text_size(self.format.text_size);
        builder.set_text_decoration(self.format.text_decoration);
        builder.set_justify_content(Justify::Center);
        builder.add_content(&self.content)?;
        builder.new_line();
        Ok(())
    }
}
