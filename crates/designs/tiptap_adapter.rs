use crate::display_utils;
use anyhow::Result;
use rongta::{
    PrintBuilder,
    elements::{Justify, TextDecoration},
};
use tiptap::{JSONContent, Mark, MarkType, NodeType};

/// Convert a textAlign attribute value to a Justify enum.
fn text_align_to_justify(align: Option<&str>) -> Justify {
    match align {
        Some("center") => Justify::Center,
        Some("right") => Justify::Right,
        _ => Justify::Left,
    }
}

pub struct TipTapJsonAdapter {
    builder: PrintBuilder,
}

impl TipTapJsonAdapter {
    pub fn new(builder: PrintBuilder) -> Self {
        Self { builder }
    }

    /// Print the Tiptap JSON content to the printer.
    ///
    /// # Arguments
    /// * `content` - The Tiptap JSON content to render
    /// * `rows` - Optional number of rows per page for paginated printing
    pub fn print(mut self, content: JSONContent, rows: Option<u32>) -> Result<()> {
        self.render_content(&content)?;
        self.builder.print(rows)?;
        log::info!("Tiptap content printed");
        Ok(())
    }
    // TODO: rendering h1 is misaligned
    // TODO: left,center right do not align 

    /// Render a Tiptap JSON node and its children.
    fn render_content(&mut self, content: &JSONContent) -> Result<()> {
        if let Some(ref node_type) = content.node_type {
            match node_type {
                NodeType::Doc => {
                    log::trace!("NodeType::Doc triggered");
                    self.render_children(content)
                }
                NodeType::Paragraph => {
                    log::trace!("NodeType::Paragraph triggered");
                    let justify = text_align_to_justify(content.paragraph_text_align());
                    self.builder.set_justify_content(justify);
                    self.render_children(content)?;
                    self.builder.new_line();
                    self.builder.set_justify_content(Justify::Left);
                    Ok(())
                }
                NodeType::Text => {
                    log::trace!("NodeType::Text triggered");
                    self.render_text(content)
                }
                NodeType::Heading => {
                    log::trace!("NodeType::Heading triggered");
                    let level = self.get_heading_level(content);
                    let justify = text_align_to_justify(content.heading_text_align());
                    let children = content.content.clone();
                    self.builder.set_justify_content(justify);
                    let result =
                        display_utils::render_heading(&mut self.builder, level, |builder| {
                            if let Some(children) = children {
                                for child in &children {
                                    Self::render_text_to_builder(builder, child)?;
                                }
                            }
                            Ok(())
                        });
                    self.builder.new_line();
                    self.builder.set_justify_content(Justify::Left);
                    result
                }
                NodeType::Blockquote => {
                    log::trace!("NodeType::Blockquote triggered");
                    let children = content.content.clone();
                    display_utils::render_blockquote(&mut self.builder, |builder| {
                        if let Some(children) = children {
                            for child in &children {
                                Self::render_node_to_builder(builder, child)?;
                            }
                        }
                        Ok(())
                    })
                }
                NodeType::BulletList => {
                    log::trace!("NodeType::BulletList triggered");
                    self.builder.new_line();
                    self.render_children(content)
                }
                NodeType::OrderedList => {
                    log::trace!("NodeType::OrderedList triggered");
                    self.builder.new_line();
                    self.render_ordered_list(content)
                }
                NodeType::ListItem => {
                    log::trace!("NodeType::ListItem triggered");
                    self.render_list_item(content, None)
                }
                NodeType::CodeBlock => {
                    log::trace!("NodeType::CodeBlock triggered");
                    let text = self.extract_text_content(content);
                    display_utils::render_code_block(&mut self.builder, &text)
                }
                NodeType::HardBreak => {
                    log::trace!("NodeType::HardBreak triggered");
                    self.builder.new_line();
                    Ok(())
                }
                NodeType::HorizontalRule => {
                    log::trace!("NodeType::HorizontalRule triggered");
                    display_utils::render_horizontal_rule(&mut self.builder)
                }
                NodeType::TaskList => {
                    log::trace!("NodeType::TaskList triggered");
                    self.builder.new_line();
                    self.render_children(content)
                }
                NodeType::TaskItem => {
                    log::trace!("NodeType::TaskItem triggered");
                    self.render_task_item(content)
                }
                NodeType::Other(name) => {
                    log::warn!("Unknown node type: {}", name);
                    Ok(())
                }
            }
        } else {
            // Node without a type - could be a fragment, render children if present
            self.render_children(content)
        }
    }

    /// Render child nodes of a content node.
    fn render_children(&mut self, content: &JSONContent) -> Result<()> {
        if let Some(ref children) = content.content {
            for child in children {
                self.render_content(child)?;
            }
        }
        Ok(())
    }

    /// Render text content with its marks applied.
    fn render_text(&mut self, content: &JSONContent) -> Result<()> {
        if let Some(ref text) = content.text {
            let decoration = self.marks_to_decoration(content.marks.as_ref());
            let has_strikethrough = self.has_strikethrough_mark(content.marks.as_ref());

            if has_strikethrough {
                self.builder.set_text_decoration(decoration);
                display_utils::render_strikethrough(&mut self.builder, text)?;
                self.builder.reset_styles();
            } else {
                self.builder.set_text_decoration(decoration);
                self.builder.add_content(text)?;
                self.builder.reset_styles();
            }
        }
        Ok(())
    }

    /// Render text content to a builder (static version for closures).
    fn render_text_to_builder(builder: &mut PrintBuilder, content: &JSONContent) -> Result<()> {
        if let Some(ref text) = content.text {
            let decoration = Self::marks_to_decoration_static(content.marks.as_ref());
            let has_strikethrough = Self::has_strikethrough_mark_static(content.marks.as_ref());

            if has_strikethrough {
                builder.set_text_decoration(decoration);
                display_utils::render_strikethrough(builder, text)?;
                builder.reset_styles();
            } else {
                builder.set_text_decoration(decoration);
                builder.add_content(text)?;
                builder.reset_styles();
            }
        }
        Ok(())
    }

    /// Render any node to a builder (static version for closures).
    fn render_node_to_builder(builder: &mut PrintBuilder, content: &JSONContent) -> Result<()> {
        if let Some(ref node_type) = content.node_type {
            match node_type {
                NodeType::Text => Self::render_text_to_builder(builder, content),
                NodeType::Paragraph => {
                    if let Some(ref children) = content.content {
                        for child in children {
                            Self::render_node_to_builder(builder, child)?;
                        }
                    }
                    builder.new_line();
                    Ok(())
                }
                _ => {
                    // For other node types within blockquote, just render text content
                    if let Some(ref children) = content.content {
                        for child in children {
                            Self::render_node_to_builder(builder, child)?;
                        }
                    }
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    /// Get the heading level from node attributes.
    fn get_heading_level(&self, content: &JSONContent) -> u8 {
        content.heading_level().unwrap_or(1)
    }

    /// Render an ordered list with numbered items.
    fn render_ordered_list(&mut self, content: &JSONContent) -> Result<()> {
        let start = content.ordered_list_start().unwrap_or(1) as u32;

        if let Some(ref children) = content.content {
            for (i, child) in children.iter().enumerate() {
                let number = start + i as u32;
                self.render_list_item(child, Some(number))?;
            }
        }
        Ok(())
    }

    /// Render a list item with optional number prefix.
    fn render_list_item(&mut self, content: &JSONContent, number: Option<u32>) -> Result<()> {
        self.builder.set_text_decoration(TextDecoration {
            bold: true,
            ..Default::default()
        });

        let prefix = match number {
            Some(n) => format!("{}. ", n),
            None => "- ".to_string(),
        };
        self.builder.add_content(&prefix)?;
        self.builder.reset_styles();

        self.render_children(content)
    }

    /// Render a task item with checkbox.
    fn render_task_item(&mut self, content: &JSONContent) -> Result<()> {
        let checked = content.task_item_checked().unwrap_or(false);

        self.builder.set_text_decoration(TextDecoration {
            bold: true,
            ..Default::default()
        });

        let prefix = display_utils::task_item_prefix(checked);
        self.builder.add_content(prefix)?;
        self.builder.reset_styles();

        self.render_children(content)
    }

    /// Convert Tiptap marks to TextDecoration.
    fn marks_to_decoration(&self, marks: Option<&Vec<Mark>>) -> TextDecoration {
        Self::marks_to_decoration_static(marks)
    }

    /// Convert Tiptap marks to TextDecoration (static version).
    fn marks_to_decoration_static(marks: Option<&Vec<Mark>>) -> TextDecoration {
        let mut decoration = TextDecoration::default();

        if let Some(marks) = marks {
            for mark in marks {
                match mark.mark_type {
                    MarkType::Bold => decoration.bold = true,
                    MarkType::Italic => decoration.underline = true, // Printer doesn't support italic
                    MarkType::Code => {
                        decoration.bold = true;
                        decoration.underline = true;
                    }
                    MarkType::Strike => {
                        // Handled separately via render_strikethrough
                    }
                    MarkType::Other(ref name) => {
                        log::warn!("Unknown mark type: {}", name);
                    }
                }
            }
        }

        decoration
    }

    /// Check if marks contain strikethrough.
    fn has_strikethrough_mark(&self, marks: Option<&Vec<Mark>>) -> bool {
        Self::has_strikethrough_mark_static(marks)
    }

    /// Check if marks contain strikethrough (static version).
    fn has_strikethrough_mark_static(marks: Option<&Vec<Mark>>) -> bool {
        marks
            .map(|m| {
                m.iter()
                    .any(|mark| matches!(mark.mark_type, MarkType::Strike))
            })
            .unwrap_or(false)
    }

    /// Extract all text content from a node and its children.
    fn extract_text_content(&self, content: &JSONContent) -> String {
        let mut text = String::new();

        if let Some(ref t) = content.text {
            text.push_str(t);
        }

        if let Some(ref children) = content.content {
            for child in children {
                text.push_str(&self.extract_text_content(child));
            }
        }

        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_text_node(text: &str) -> JSONContent {
        JSONContent {
            node_type: Some(NodeType::Text),
            attrs: None,
            content: None,
            marks: None,
            text: Some(text.to_string()),
            extra: HashMap::new(),
        }
    }

    fn create_paragraph(children: Vec<JSONContent>) -> JSONContent {
        JSONContent {
            node_type: Some(NodeType::Paragraph),
            attrs: None,
            content: Some(children),
            marks: None,
            text: None,
            extra: HashMap::new(),
        }
    }

    fn create_doc(children: Vec<JSONContent>) -> JSONContent {
        JSONContent {
            node_type: Some(NodeType::Doc),
            attrs: None,
            content: Some(children),
            marks: None,
            text: None,
            extra: HashMap::new(),
        }
    }

    #[test]
    fn test_marks_to_decoration_empty() {
        let decoration = TipTapJsonAdapter::marks_to_decoration_static(None);
        assert!(!decoration.bold);
        assert!(!decoration.underline);
        assert!(!decoration.italic);
    }

    #[test]
    fn test_marks_to_decoration_bold() {
        let marks = vec![Mark {
            mark_type: MarkType::Bold,
            attrs: None,
            extra: HashMap::new(),
        }];
        let decoration = TipTapJsonAdapter::marks_to_decoration_static(Some(&marks));
        assert!(decoration.bold);
        assert!(!decoration.underline);
    }

    #[test]
    fn test_marks_to_decoration_italic() {
        let marks = vec![Mark {
            mark_type: MarkType::Italic,
            attrs: None,
            extra: HashMap::new(),
        }];
        let decoration = TipTapJsonAdapter::marks_to_decoration_static(Some(&marks));
        assert!(!decoration.bold);
        assert!(decoration.underline); // Italic maps to underline
    }

    #[test]
    fn test_marks_to_decoration_code() {
        let marks = vec![Mark {
            mark_type: MarkType::Code,
            attrs: None,
            extra: HashMap::new(),
        }];
        let decoration = TipTapJsonAdapter::marks_to_decoration_static(Some(&marks));
        assert!(decoration.bold);
        assert!(decoration.underline);
    }

    #[test]
    fn test_marks_to_decoration_combined() {
        let marks = vec![
            Mark {
                mark_type: MarkType::Bold,
                attrs: None,
                extra: HashMap::new(),
            },
            Mark {
                mark_type: MarkType::Italic,
                attrs: None,
                extra: HashMap::new(),
            },
        ];
        let decoration = TipTapJsonAdapter::marks_to_decoration_static(Some(&marks));
        assert!(decoration.bold);
        assert!(decoration.underline);
    }

    #[test]
    fn test_has_strikethrough_mark_true() {
        let marks = vec![Mark {
            mark_type: MarkType::Strike,
            attrs: None,
            extra: HashMap::new(),
        }];
        assert!(TipTapJsonAdapter::has_strikethrough_mark_static(Some(
            &marks
        )));
    }

    #[test]
    fn test_has_strikethrough_mark_false() {
        let marks = vec![Mark {
            mark_type: MarkType::Bold,
            attrs: None,
            extra: HashMap::new(),
        }];
        assert!(!TipTapJsonAdapter::has_strikethrough_mark_static(Some(
            &marks
        )));
    }

    #[test]
    fn test_has_strikethrough_mark_none() {
        assert!(!TipTapJsonAdapter::has_strikethrough_mark_static(None));
    }

    #[test]
    fn test_extract_text_content_simple() {
        let builder = PrintBuilder::new(false);
        let adapter = TipTapJsonAdapter::new(builder);

        let content = create_text_node("Hello");
        assert_eq!(adapter.extract_text_content(&content), "Hello");
    }

    #[test]
    fn test_extract_text_content_nested() {
        let builder = PrintBuilder::new(false);
        let adapter = TipTapJsonAdapter::new(builder);

        let content = create_doc(vec![create_paragraph(vec![
            create_text_node("Hello "),
            create_text_node("World"),
        ])]);

        assert_eq!(adapter.extract_text_content(&content), "Hello World");
    }

    #[test]
    fn test_get_heading_level_default() {
        let builder = PrintBuilder::new(false);
        let adapter = TipTapJsonAdapter::new(builder);

        let content = JSONContent {
            node_type: Some(NodeType::Heading),
            attrs: None,
            content: None,
            marks: None,
            text: None,
            extra: HashMap::new(),
        };

        assert_eq!(adapter.get_heading_level(&content), 1);
    }

    #[test]
    fn test_get_heading_level_from_attrs() {
        let builder = PrintBuilder::new(false);
        let adapter = TipTapJsonAdapter::new(builder);

        let mut attrs = HashMap::new();
        attrs.insert("level".to_string(), serde_json::json!(2));

        let content = JSONContent {
            node_type: Some(NodeType::Heading),
            attrs: Some(attrs),
            content: None,
            marks: None,
            text: None,
            extra: HashMap::new(),
        };

        assert_eq!(adapter.get_heading_level(&content), 2);
    }
}
