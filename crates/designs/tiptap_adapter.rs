use crate::render;
use anyhow::Result;
use rongta::{PrintBuilder, ToBuilderCommand, elements::Justify};
use tiptap::{JSONContent, NodeType};

pub struct TipTapJsonAdapter {
    builder: PrintBuilder,
}

impl TipTapJsonAdapter {
    pub fn new(builder: PrintBuilder) -> Self {
        Self { builder }
    }

    pub fn print(mut self, content: JSONContent, rows: Option<u32>) -> Result<()> {
        self.render_content(&content)?;
        self.builder.print(rows)?;
        log::info!("Tiptap content printed");
        Ok(())
    }

    fn render_content(&mut self, content: &JSONContent) -> Result<()> {
        if let Some(ref node_type) = content.node_type {
            match node_type {
                NodeType::Doc => {
                    log::trace!("NodeType::Doc triggered");
                    if let Some(content) = &content.content {
                        for child in content {
                            self.render_content(&child)?;
                        }
                    }
                    Ok(())
                }
                NodeType::Paragraph => {
                    log::trace!("NodeType::Paragraph triggered");
                    self.builder.new_line();
                    if let Some(alignment) = content.text_align() {
                        match alignment {
                            tiptap::TextAlign::Left => {
                                self.builder.set_justify_content(Justify::Left);
                            }
                            tiptap::TextAlign::Center => {
                                self.builder.set_justify_content(Justify::Center);
                            }
                            tiptap::TextAlign::Right => {
                                self.builder.set_justify_content(Justify::Center);
                            }
                        }
                    }
                    if let Some(content) = &content.content {
                        for child in content {
                            self.render_content(&child)?;
                        }
                    }
                    Ok(())
                }
                NodeType::Text => {
                    log::trace!("NodeType::Text triggered");
                    if let Some(text) = content.text.clone() {
                        let command = render::Text::new(text, None, Some(content.is_bold()));
                        command.to_builder_command(&mut self.builder)
                    } else {
                        Ok(())
                    }
                }
                NodeType::Heading => {
                    log::trace!("NodeType::Heading triggered");
                    if let Some(inner_text) = get_inner_text(content) {
                        let command = render::Heading::new(inner_text, content.heading_level());
                        command.to_builder_command(&mut self.builder)
                    } else {
                        Ok(())
                    }
                }
                NodeType::Blockquote => {
                    log::trace!("NodeType::Blockquote triggered");
                    if let Some(inner_text) = get_inner_text(content) {
                        let command = render::BlockQuote::new(inner_text);
                        command.to_builder_command(&mut self.builder)
                    } else {
                        Ok(())
                    }
                }
                NodeType::BulletList => {
                    log::trace!("NodeType::BulletList triggered");
                    let command = render::ListItemBefore::new_unordered();
                    command.to_builder_command(&mut self.builder)
                }
                NodeType::OrderedList => {
                    log::trace!("NodeType::OrderedList triggered");
                    let command = render::ListItemBefore::new_ordered(
                        content.ordered_list_start(),
                        content.ordered_list_type(),
                    );
                    command.to_builder_command(&mut self.builder)
                }
                NodeType::ListItem => {
                    log::trace!("NodeType::ListItem triggered");
                    if let Some(inner_text) = get_inner_text(content) {
                        let command = render::Text::new(inner_text, None, None);
                        command.to_builder_command(&mut self.builder)
                    } else {
                        Ok(())
                    }
                }
                NodeType::CodeBlock => {
                    log::trace!("NodeType::CodeBlock triggered");
                    if let Some(inner_text) = get_inner_text(content) {
                        let command = render::CodeBlock::new(inner_text);
                        command.to_builder_command(&mut self.builder)
                    } else {
                        Ok(())
                    }
                }
                NodeType::HardBreak => {
                    log::trace!("NodeType::HardBreak triggered");
                    self.builder.new_line();
                    Ok(())
                }
                NodeType::HorizontalRule => {
                    log::trace!("NodeType::HorizontalRule triggered");
                    let command = render::HorizontalRule::new();
                    command.to_builder_command(&mut self.builder)
                }
                NodeType::TaskList => {
                    log::trace!("NodeType::TaskList triggered");
                    let command =
                        render::TaskListBefore::new(content.is_checked().unwrap_or_default());
                    command.to_builder_command(&mut self.builder)
                }
                NodeType::TaskItem => {
                    log::trace!("NodeType::TaskItem triggered");
                    if let Some(inner_text) = get_inner_text(content) {
                        let command = render::Text::new(inner_text, None, None);
                        command.to_builder_command(&mut self.builder)
                    } else {
                        Ok(())
                    }
                }
                NodeType::Other(name) => {
                    log::warn!("Unknown node type: {}", name);
                    Ok(())
                }
            }
        } else {
            // Node without a type - could be a fragment, render children if present
            self.render_content(content)
        }
    }
}

fn get_inner_text(content: &JSONContent) -> Option<String> {
    if content.node_type == Some(NodeType::Text) {
        return content.text.clone();
    } else {
        let mut inner_text = "".to_string();
        for child in content.content.as_ref().unwrap_or(&vec![]) {
            if let Some(i_text) = get_inner_text(&child) {
                inner_text.push_str(&i_text);
            };
        }
        if inner_text.trim().len() > 0 {
            return Some(inner_text);
        } else {
            None
        }
    }
}
