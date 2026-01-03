use anyhow::Result;
use comrak::nodes::{AstNode, NodeValue};
use rongta::{PrintBuilder, TextDecoration, TextSize};

pub struct MarkdownFileAdapter {
    builder: PrintBuilder,
}
impl MarkdownFileAdapter {
    pub fn new(builder: PrintBuilder) -> Self {
        Self { builder }
    }
    pub fn print(&mut self, content: &str, rows: Option<u32>) -> Result<()> {
        let arena = comrak::Arena::new();
        let mut options = comrak::Options::default();
        options.parse.smart = true;
        options.extension.strikethrough = true;
        options.extension.tasklist = true;
        let root = comrak::parse_document(&arena, content, &options);
        self.render_node(root)?;
        self.builder.print(rows)?;
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
                self.builder.new_line();
                self.builder.new_line();
                self.builder.set_text_decoration(TextDecoration {
                    bold: true,
                    italic: false,
                    underline: true,
                });
                self.builder.set_justify_content(rongta::Justify::Center);
                self.render_children(node)?;
                self.builder.new_line();
                self.builder.new_line();
                self.builder.reset_styles();
                Ok(())
            }
            NodeValue::List(node_list) => {
                log::trace!("NodeValue::List triggered");
                self.builder.new_line();
                self.render_children(node)?;
                Ok(())
            }
            NodeValue::Item(node_list) => {
                log::trace!("NodeValue::Item triggered");
                self.builder.set_text_decoration(TextDecoration {
                    bold: true,
                    ..Default::default()
                });
                match node_list.list_type {
                    comrak::nodes::ListType::Bullet => self.builder.add_content("- "),
                    comrak::nodes::ListType::Ordered => self.builder.add_content(&format!(
                        "{}{} ",
                        node_list.start,
                        match node_list.delimiter {
                            comrak::nodes::ListDelimType::Period => '.',
                            comrak::nodes::ListDelimType::Paren => ')',
                        }
                    )),
                }?;
                self.builder.reset_styles();
                self.render_children(node)
            }
            NodeValue::CodeBlock(node_code_block) => {
                log::trace!("NodeValue::CodeBlock triggered");
                self.builder.new_line();
                self.builder.new_line();
                self.builder.set_text_decoration(TextDecoration {
                    bold: true,
                    ..Default::default()
                });
                self.builder.add_content(&node_code_block.literal)?;
                self.builder.new_line();
                self.builder.new_line();
                self.builder.reset_styles();
                Ok(())
            }
            NodeValue::Paragraph => {
                log::trace!("NodeValue::Paragraph triggered");
                self.render_children(node)?;
                self.builder.new_line();
                Ok(())
            }
            NodeValue::Heading(node_heading) => {
                log::trace!(
                    "NodeValue::Heading triggered (level: {})",
                    node_heading.level
                );
                let (size, decoration) = match node_heading.level {
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
                };
                self.builder.new_line();
                self.builder.set_text_size(size);
                self.builder.set_text_decoration(decoration);
                self.builder.set_justify_content(rongta::Justify::Center);
                self.render_children(node)?;
                self.builder.reset_styles();
                Ok(())
            }
            NodeValue::Text(cow) => {
                log::trace!("Text: {}", cow);
                self.builder.add_content(cow)
            }
            NodeValue::TaskItem(node_task_item) => {
                log::trace!("NodeValue::TaskItem triggered");
                self.builder.set_text_decoration(TextDecoration {
                    bold: true,
                    ..Default::default()
                });
                let prefix = if node_task_item.symbol.is_some() {
                    "[x] "
                } else {
                    "[ ] "
                };
                self.builder.add_content(prefix)?;
                self.builder.reset_styles();
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
            NodeValue::Code(node_code) => {
                log::trace!("NodeValue::Code triggered");
                self.builder.set_text_decoration(TextDecoration {
                    bold: true,
                    underline: true,
                    ..Default::default()
                });
                self.builder.add_content(&node_code.literal)?;
                self.builder.reset_styles();
                Ok(())
            }
            NodeValue::Emph => {
                log::trace!("NodeValue::Emph triggered");
                self.builder.set_text_decoration(TextDecoration {
                    underline: true,
                    ..Default::default()
                });
                self.render_children(node)?;
                self.builder.reset_styles();
                Ok(())
            }
            NodeValue::Strong => {
                log::trace!("NodeValue::Strong triggered");
                self.builder.set_text_decoration(TextDecoration {
                    bold: true,
                    ..Default::default()
                });
                self.render_children(node)?;
                self.builder.reset_styles();
                Ok(())
            }
            NodeValue::Strikethrough => {
                log::trace!("NodeValue::Strikethrough triggered");
                self.builder.add_content("--")?;
                self.render_children(node)?;
                self.builder.add_content("--")
            }
            NodeValue::Link(node_link) => {
                log::trace!("NodeValue::Link triggered");
                self.builder.add_content(&node_link.title)
            }
            NodeValue::Image(node_link) => {
                log::trace!("NodeValue::Image triggered");
                self.builder.new_line();
                self.builder.set_justify_content(rongta::Justify::Center);
                self.builder.add_content(&node_link.title)?;
                self.builder.new_line();
                self.builder.reset_styles();
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
