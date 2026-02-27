use crate::interpreter::block_adornment::{HorizontalRule, ListItemBefore, TaskListBefore};
use anyhow::Result;
use pulldown_cmark::{Options, Parser, Tag};
use rongta::{RongtaPrinter, SupportedDriver, ToBuilderCommand};

pub struct MarkdownInterpreter {
    builder: RongtaPrinter,
    list_index: Option<u64>,
}
impl MarkdownInterpreter {
    pub fn new(builder: RongtaPrinter) -> Self {
        Self {
            builder,
            list_index: None,
        }
    }

    pub fn print(
        &mut self,
        content: &str,
        rows: Option<u32>,
        driver: SupportedDriver,
    ) -> Result<()> {
        self.render_content(content)?;
        self.builder.print(rows, driver)?;
        log::info!("Markdown content printed");
        Ok(())
    }

    fn handle_tag_start(&mut self, tag: &Tag) -> Result<()> {
        match tag {
            Tag::Paragraph => {
                self.builder.reset_styles();
                Ok(())
            },
            Tag::Heading {
                level,
                id: _,
                classes: _,
                attrs: _,
            } => {
                let level = match level {
                    pulldown_cmark::HeadingLevel::H1 => 1,
                    pulldown_cmark::HeadingLevel::H2 => 2,
                    pulldown_cmark::HeadingLevel::H3 => 3,
                    pulldown_cmark::HeadingLevel::H4 => 4,
                    pulldown_cmark::HeadingLevel::H5 => 5,
                    pulldown_cmark::HeadingLevel::H6 => 6,
                };
                super::block_adornment::set_heading_style(level, &mut self.builder)
            }
            Tag::BlockQuote(_) | Tag::CodeBlock(_) => {
                self.builder.new_line();
                self.builder.reset_styles();
                self.builder.set_is_bold(true);
                Ok(())
            }
            Tag::List(ordered_start) => {
                self.list_index = *ordered_start;
                Ok(())
            }
            Tag::Item => {
                let before = match self.list_index {
                    Some(i) => {
                        let mut b = ListItemBefore::new_ordered(None);
                        b.next_index(i);
                        b
                    }
                    None => ListItemBefore::new_unordered(),
                };
                before.to_builder_command(&mut self.builder)
            }
            Tag::Strong => {
                self.builder.set_is_bold(true);
                Ok(())
            },
            // Tag::Strikethrough => todo!(),
            // Tag::Link {
            //     link_type,
            //     dest_url,
            //     title,
            //     id,
            // } => todo!(),
            // Tag::Image {
            //     link_type,
            //     dest_url,
            //     title,
            //     id,
            // } => todo!(),
            _ => Ok(()),
        }
    }

    fn render_content(&mut self, markdown: &str) -> Result<()> {
        for event in Parser::new_ext(markdown, Options::ENABLE_TASKLISTS) {
            match &event {
                pulldown_cmark::Event::Start(tag) => self.handle_tag_start(tag),
                pulldown_cmark::Event::End(_) => {
                    self.builder.new_line();
                    continue;
                }
                pulldown_cmark::Event::Text(cow_str) => self.builder.add_content(cow_str),
                pulldown_cmark::Event::Code(_)
                | pulldown_cmark::Event::InlineMath(_)
                | pulldown_cmark::Event::DisplayMath(_)
                | pulldown_cmark::Event::Html(_)
                | pulldown_cmark::Event::InlineHtml(_)
                | pulldown_cmark::Event::FootnoteReference(_) => continue,
                pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                    self.builder.new_line();
                    continue;
                }
                pulldown_cmark::Event::Rule => {
                    let r = HorizontalRule::new();
                    r.to_builder_command(&mut self.builder)
                }
                pulldown_cmark::Event::TaskListMarker(checked) => {
                    let before = TaskListBefore::new(*checked);
                    before.to_builder_command(&mut self.builder)
                }
            }?;
        }
        Ok(())
    }
}
