use super::block_adornment::{HorizontalRule, ListItemBefore, TaskListBefore};
use crate::interpreter::block_adornment::{self, ToBuilderCommand};
use anyhow::Result;
use rongta::{RongtaPrinter, SupportedDriver, elements::Justify};
use tiptap::{NodeType, TipTapNode};

pub struct TipTapInterpreter {
    builder: RongtaPrinter,
    list: Option<ListItemBefore>,
    list_start: Option<u64>,
}
impl TipTapInterpreter {
    pub fn new(builder: RongtaPrinter) -> Self {
        Self {
            builder,
            list: None,
            list_start: None,
        }
    }

    pub fn print(
        &mut self,
        content: TipTapNode,
        rows: Option<u32>,
        driver: SupportedDriver,
    ) -> Result<()> {
        self.render_content(&content)?;
        self.builder.print(rows, driver)?;
        log::info!("Tiptap content printed");
        Ok(())
    }

    fn handle_text_align_attribute(&mut self, node: &TipTapNode) -> Result<()> {
        if let Some(alignment) = node.text_align() {
            log::trace!("Found alignment");
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

    fn handle_bold_mark(&mut self, node: &TipTapNode) -> Result<()> {
        self.builder.set_is_bold(node.is_bold());
        Ok(())
    }

    fn handle_heading_style(&mut self, node: &TipTapNode) -> Result<()> {
        let level = node.heading_level().unwrap_or(3);
        block_adornment::set_heading_style(level, &mut self.builder)
    }

    fn render_content(&mut self, node: &TipTapNode) -> Result<()> {
        for event in node.events() {
            match event {
                tiptap::Event::NodeStart(tip_tap_node) => match tip_tap_node.node_type {
                    NodeType::Doc => continue,
                    NodeType::Paragraph => {
                        self.builder.new_line();
                        self.builder.reset_styles();
                        self.handle_text_align_attribute(tip_tap_node)?
                    }
                    NodeType::Text => self.handle_bold_mark(tip_tap_node)?,
                    NodeType::Heading => {
                        self.handle_text_align_attribute(tip_tap_node)?;
                        self.handle_heading_style(tip_tap_node)?;
                    }
                    NodeType::BulletList => {
                        self.list = Some(ListItemBefore::new_unordered());
                        self.list_start = None;
                    }
                    NodeType::OrderedList => {
                        self.list = Some(ListItemBefore::new_ordered(
                            tip_tap_node.ordered_list_type(),
                        ));
                        self.list_start = tip_tap_node.ordered_list_start()
                    }
                    NodeType::ListItem => {
                        if let Some(ref list) = self.list {
                            let mut before = list.clone();
                            if let Some(ref mut start) = self.list_start {
                                before.next_index(*start);
                                *start += 1;
                            }
                            before.to_builder_command(&mut self.builder)?;
                        }
                        continue;
                    }
                    NodeType::CodeBlock => {
                        self.builder.new_line();
                        self.builder.set_is_bold(true);
                    }
                    NodeType::HardBreak => self.builder.new_line(),
                    NodeType::HorizontalRule => {
                        self.builder.new_line();
                        let line = HorizontalRule::new();
                        line.to_builder_command(&mut self.builder)?;
                    }
                    NodeType::TaskList => self.builder.new_line(),
                    NodeType::TaskItem => {
                        let before =
                            TaskListBefore::new(tip_tap_node.is_checked().unwrap_or_default());
                        before.to_builder_command(&mut self.builder)?;
                    }
                },
                tiptap::Event::NodeEnd(tip_tap_node) => match tip_tap_node.node_type {
                    NodeType::Doc => break,
                    NodeType::ListItem => self.builder.new_line(),
                    NodeType::TaskItem => self.builder.new_line(),
                    _ => {
                        self.builder.new_line();
                        self.builder.reset_styles();
                    }
                },
                tiptap::Event::Text(content, _) => self.builder.add_content(content)?,
            };
        }
        Ok(())
    }
}
