use anyhow::Result;
use rongta::{
    RongtaPrinter, ToBuilderCommand,
    elements::{FormatState, Justify, TextSize},
};
use tiptap::OrderedListType;

/// Style the ListItem ::before pseudoelement
pub struct ListItemBefore {
    ordinal: Option<OrderedListType>,
    content: String,
    format: FormatState,
}
impl ListItemBefore {
    pub fn new_ordered(ordinal: Option<OrderedListType>) -> Self {
        Self {
            content: "".to_string(),
            ordinal,
            format: FormatState {
                text_size: TextSize::Medium,
                is_bold: true,
            },
        }
    }
    pub fn new_unordered() -> Self {
        Self {
            content: "∙ ".to_string(),
            format: FormatState {
                text_size: TextSize::Medium,
                is_bold: true,
            },
            ordinal: None,
        }
    }
    fn ordered_before_content(index: u64, ordinal: &Option<OrderedListType>) -> String {
        let value = match ordinal.clone().unwrap_or_default() {
            OrderedListType::LowerCaseLetter => Self::letter_for_index(index, false),
            OrderedListType::UpperCaseLetter => Self::letter_for_index(index, true),
            OrderedListType::LowerCaseRoman => Self::roman_numeral(index, false),
            OrderedListType::UpperCaseRoman => Self::roman_numeral(index, true),
            OrderedListType::Number => index.to_string(),
        };
        format!("{}. ", value)
    }
    pub fn next_index(&mut self, index: u64) {
        self.content = Self::ordered_before_content(index, &self.ordinal);
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
        builder.set_is_bold(self.format.is_bold);
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
            "[x] ".to_string()
        } else {
            "[ ] ".to_string()
        };
        Self {
            content,
            format: FormatState {
                text_size: TextSize::Medium,
                is_bold: true,
            },
        }
    }
}
impl ToBuilderCommand for TaskListBefore {
    fn to_builder_command(&self, builder: &mut RongtaPrinter) -> Result<()> {
        builder.new_line();
        builder.reset_styles();
        builder.set_text_size(self.format.text_size);
        builder.set_is_bold(self.format.is_bold);
        builder.add_content(&self.content)
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
                is_bold: true,
            },
        }
    }
}
impl ToBuilderCommand for HorizontalRule {
    fn to_builder_command(&self, builder: &mut RongtaPrinter) -> Result<()> {
        builder.new_line();
        builder.reset_styles();
        builder.set_text_size(self.format.text_size);
        builder.set_is_bold(self.format.is_bold);
        builder.set_justify_content(Justify::Center);
        builder.add_content(&self.content)?;
        builder.new_line();
        Ok(())
    }
}
