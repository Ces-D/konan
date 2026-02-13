use crate::render::{HorizontalRule, ListItemBefore, TaskListBefore};
use anyhow::{Result, bail};
use rongta::{
    RongtaPrinter, SupportedDriver, ToBuilderCommand,
    elements::{Justify, TextSize},
};
use tiptap::{JSONContent, NodeType};

pub struct TipTapInterpreter {
    builder: RongtaPrinter,
}
impl TipTapInterpreter {
    pub fn new(builder: RongtaPrinter) -> Self {
        Self { builder }
    }

    pub fn print(
        mut self,
        content: JSONContent,
        rows: Option<u32>,
        driver: SupportedDriver,
    ) -> Result<()> {
        self.render_content(&content)?;
        self.builder.print(rows, driver)?;
        log::info!("Tiptap content printed");
        Ok(())
    }

    fn handle_text_align_attribute(&mut self, node: &JSONContent) -> Result<()> {
        if let Some(alignment) = node.text_align() {
            let justification = match alignment {
                tiptap::TextAlign::Left => Justify::Left,
                tiptap::TextAlign::Center => Justify::Center,
                tiptap::TextAlign::Right => Justify::Right,
            };
            self.builder.set_justify_content(justification);
        } else {
            self.builder.set_justify_content(Justify::Left);
        }
        Ok(())
    }

    fn handle_bold_mark(&mut self, node: &JSONContent) -> Result<()> {
        self.builder.set_is_bold(node.is_bold());
        Ok(())
    }

    fn handle_heading_style(&mut self, node: &JSONContent) -> Result<()> {
        let level = node.heading_level().unwrap_or(3);
        match level {
            1 => {
                self.builder.set_text_size(TextSize::ExtraLarge);
                self.builder.set_is_bold(true);
            }
            2 => {
                self.builder.set_text_size(TextSize::Large);
                self.builder.set_is_bold(true);
            }
            3 => {
                self.builder.set_text_size(TextSize::Large);
                self.builder.set_is_bold(false);
            }
            _ => {
                self.builder.set_text_size(TextSize::Medium);
                self.builder.set_is_bold(true);
            }
        };

        Ok(())
    }

    fn render_content(&mut self, node: &JSONContent) -> Result<()> {
        match node.node_type.as_ref() {
            Some(ntype) => match ntype {
                NodeType::Doc => {
                    log::trace!("NodeType::Doc triggered");
                    self.render_children(node)
                }
                NodeType::Paragraph => {
                    self.handle_text_align_attribute(node)?;
                    self.render_children(node)?;
                    Ok(())
                }
                NodeType::Text => {
                    self.handle_bold_mark(node)?;
                    if let Some(text) = &node.text {
                        self.builder.add_content(text)?;
                    }
                    Ok(())
                }
                NodeType::Heading => {
                    self.builder.new_line();
                    self.handle_text_align_attribute(node)?;
                    self.handle_heading_style(node)?;
                    if let Some(children) = &node.content {
                        // necessary to maintain reinforced heading style
                        for child in children {
                            if let Some(text) = &child.text {
                                self.builder.add_content(text)?;
                            }
                        }
                    }
                    self.builder.reset_styles();
                    self.builder.new_line();
                    Ok(())
                }
                NodeType::BulletList => {
                    self.builder.new_line();
                    let before = ListItemBefore::new_unordered();
                    if let Some(children) = &node.content {
                        for child in children {
                            before.to_builder_command(&mut self.builder)?;
                            self.render_content(child)?;
                        }
                    }
                    self.builder.reset_styles();
                    Ok(())
                }
                NodeType::OrderedList => {
                    self.builder.new_line();
                    let mut before = ListItemBefore::new_ordered(node.ordered_list_type());
                    if let Some(children) = &node.content {
                        for (index, child) in children.iter().enumerate() {
                            before.next_index((index as u64) + 1);
                            before.to_builder_command(&mut self.builder)?;
                            self.render_content(child)?;
                        }
                    }
                    self.builder.reset_styles();
                    Ok(())
                }
                NodeType::ListItem => self.render_children(node),
                NodeType::TaskList => {
                    self.builder.new_line();
                    if let Some(children) = &node.content {
                        for child in children {
                            let before = TaskListBefore::new(node.is_checked().unwrap_or_default());
                            before.to_builder_command(&mut self.builder)?;
                            self.render_content(child)?;
                        }
                    }
                    self.builder.reset_styles();
                    Ok(())
                }
                NodeType::TaskItem => self.render_children(node),
                NodeType::CodeBlock => {
                    self.builder.new_line();
                    self.builder.new_line();
                    self.builder.set_is_bold(true);
                    self.render_children(node)?;
                    self.builder.new_line();
                    self.builder.new_line();
                    self.builder.reset_styles();
                    Ok(())
                }
                NodeType::HardBreak => {
                    self.builder.new_line();
                    Ok(())
                }
                NodeType::HorizontalRule => {
                    let line = HorizontalRule::new();
                    line.to_builder_command(&mut self.builder)?;
                    Ok(())
                }
            },
            None => bail!("Node without a node type"),
        }
    }

    fn render_children(&mut self, node: &JSONContent) -> Result<()> {
        if let Some(content) = &node.content {
            for child in content {
                self.render_content(&child)?;
            }
        }
        Ok(())
    }
}
