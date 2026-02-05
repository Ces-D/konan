use crate::render;
use anyhow::Result;
use comrak::nodes::{AstNode, NodeValue};
use rongta::{RongtaPrinter, SupportedDriver, ToBuilderCommand};

pub struct MarkdownFileAdapter {
    builder: RongtaPrinter,
}
impl MarkdownFileAdapter {
    pub fn new(builder: RongtaPrinter) -> Self {
        Self { builder }
    }
    pub fn print(
        &mut self,
        content: &str,
        rows: Option<u32>,
        driver: SupportedDriver,
    ) -> Result<()> {
        let mut options = comrak::Options::default();
        let arena = comrak::Arena::new();
        options.parse.smart = true;
        options.extension.strikethrough = true;
        options.extension.tasklist = true;
        let root = comrak::parse_document(&arena, content, &options);
        self.render_node(root)?;
        self.builder.print(rows, driver)?;
        log::info!("Markdown file printed");
        Ok(())
    }
    /// Adapter logic for a markdown node into Rongta  
    fn render_node<'a>(&mut self, node: &'a AstNode<'a>) -> Result<()> {
        match &node.data().value {
            NodeValue::Document => {
                log::trace!("NodeValue::Document triggered");
                self.render_children(node)
            }
            NodeValue::BlockQuote => {
                log::trace!("NodeValue::BlockQuote triggered");
                let inner_text = get_inner_text(node);
                let command = render::BlockQuote::new(inner_text);
                command.to_builder_command(&mut self.builder)?;
                self.render_children(node)
            }
            NodeValue::List(node_list) => {
                log::trace!("NodeValue::List triggered");
                match node_list.list_type {
                    comrak::nodes::ListType::Bullet => {
                        let command = render::ListItemBefore::new_unordered();
                        command.to_builder_command(&mut self.builder)?;
                        self.render_children(node)
                    }
                    comrak::nodes::ListType::Ordered => {
                        let command =
                            render::ListItemBefore::new_ordered(Some(node_list.start as u64), None);
                        command.to_builder_command(&mut self.builder)?;
                        self.render_children(node)
                    }
                }
            }
            NodeValue::Item(_) => {
                log::trace!("NodeValue::Item triggered");
                let inner_text = get_inner_text(node);
                let command = render::Text::new(inner_text, None, None);
                command.to_builder_command(&mut self.builder)
            }
            NodeValue::CodeBlock(_) => {
                log::trace!("NodeValue::CodeBlock triggered");
                let inner_text = get_inner_text(node);
                let command = render::CodeBlock::new(inner_text);
                command.to_builder_command(&mut self.builder)
            }
            NodeValue::Paragraph => {
                log::trace!("NodeValue::Paragraph triggered");
                self.builder.new_line();
                self.render_children(node)
            }
            NodeValue::Heading(node_heading) => {
                log::trace!(
                    "NodeValue::Heading triggered (level: {})",
                    node_heading.level
                );
                let inner_text = get_inner_text(node);
                let command = render::Heading::new(inner_text, Some(node_heading.level));
                command.to_builder_command(&mut self.builder)
            }
            NodeValue::Text(cow) => {
                log::trace!("Text: {}", cow);
                let command = render::Text::new(cow.to_string(), None, None);
                command.to_builder_command(&mut self.builder)
            }
            NodeValue::TaskItem(node_task_item) => {
                log::trace!("NodeValue::TaskItem triggered");
                let command = render::TaskListBefore::new(node_task_item.symbol.is_some());
                command.to_builder_command(&mut self.builder)?;
                self.render_children(node)
            }
            NodeValue::SoftBreak => {
                log::trace!("NodeValue::SoftBreak triggered");
                self.builder.new_line();
                Ok(())
            }
            NodeValue::LineBreak => {
                log::trace!("NodeValue::LineBreak triggered");
                self.builder.new_line();
                self.builder.new_line();
                Ok(())
            }
            // Inline
            NodeValue::Code(_) => {
                log::trace!("NodeValue::Code triggered");
                self.render_children(node)
            }
            NodeValue::Emph => {
                log::trace!("NodeValue::Emph triggered");
                let inner_text = get_inner_text(node);
                let command = render::Text::new(inner_text, None, Some(true));
                command.to_builder_command(&mut self.builder)
            }
            NodeValue::Strong => {
                log::trace!("NodeValue::Strong triggered");
                let inner_text = get_inner_text(node);
                let command = render::Text::new(inner_text, None, Some(true));
                command.to_builder_command(&mut self.builder)
            }
            NodeValue::Strikethrough => {
                log::trace!("NodeValue::Strikethrough triggered");
                let inner_text = get_inner_text(node);
                self.builder.add_content("--")?;
                let command = render::Text::new(inner_text, None, Some(true));
                command.to_builder_command(&mut self.builder)?;
                self.builder.add_content("--")
            }
            NodeValue::Link(node_link) => {
                log::trace!("NodeValue::Link triggered");
                let command = render::Text::new(node_link.title.clone(), None, Some(true));
                command.to_builder_command(&mut self.builder)
            }
            NodeValue::Image(node_link) => {
                log::trace!("NodeValue::Image triggered");
                self.builder.new_line();
                self.builder
                    .set_justify_content(rongta::elements::Justify::Center);
                let command = render::Text::new(node_link.title.clone(), None, Some(true));
                command.to_builder_command(&mut self.builder)?;
                self.builder.new_line();
                Ok(())
            }
            _ => self.render_children(node), // NodeValue::FrontMatter(_) => todo!(),
                                             // NodeValue::HtmlInline(_) => todo!(),
                                             // NodeValue::HeexInline(_) => todo!(),
                                             // NodeValue::Raw(_) => todo!(),
                                             // NodeValue::Highlight => todo!(),
                                             // NodeValue::Superscript => todo!(),
                                             // NodeValue::FootnoteReference(node_footnote_reference) => todo!(),
                                             // NodeValue::ShortCode(node_short_code) => todo!(),
                                             // NodeValue::Math(node_math) => todo!(),
                                             // NodeValue::MultilineBlockQuote(node_multiline_block_quote) => todo!(),
                                             // NodeValue::Escaped => todo!(),
                                             // NodeValue::WikiLink(node_wiki_link) => todo!(),
                                             // NodeValue::Underline => todo!(),
                                             // NodeValue::Subscript => todo!(),
                                             // NodeValue::SpoileredText => todo!(),
                                             // NodeValue::EscapedTag(_) => todo!(),
                                             // NodeValue::Alert(node_alert) => todo!(),
                                             // NodeValue::Subtext => todo!(),
                                             // NodeValue::ThematicBreak => todo!(),
                                             // NodeValue::FootnoteDefinition(node_footnote_definition) => todo!(),
                                             // NodeValue::Table(node_table) => todo!(),
                                             // NodeValue::TableRow(_) => todo!(),
                                             // NodeValue::TableCell => todo!(),
                                             // NodeValue::HtmlBlock(node_html_block) => todo!(),
                                             // NodeValue::HeexBlock(node_heex_block) => todo!(),
                                             // NodeValue::DescriptionList => todo!(),
                                             // NodeValue::DescriptionItem(node_description_item) => todo!(),
                                             //  NodeValue::DescriptionTerm => todo!(),
                                             // NodeValue::DescriptionDetails => todo!(),
        }
    }
    /// Render the children of a document
    fn render_children<'a>(&mut self, node: &'a AstNode<'a>) -> Result<()> {
        for child in node.children() {
            self.render_node(child)?;
        }
        Ok(())
    }
}

/// Only goes one level deep in search of text
fn get_inner_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut inner_text = String::new();
    for child in node.children() {
        match &child.data().value {
            NodeValue::Text(cow) => inner_text.push_str(&cow.to_string()),
            _ => continue,
        }
    }
    inner_text
}
