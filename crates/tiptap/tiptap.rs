use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NodeType {
    /// The root document node
    Doc,
    /// A paragraph block
    Paragraph,
    /// Inline text content
    Text,
    /// Heading levels 1-4
    Heading,
    /// An unordered (bullet) list
    BulletList,
    /// An ordered (numbered) list
    OrderedList,
    /// A list item within bulletList or orderedList
    ListItem,
    /// A fenced code block
    CodeBlock,
    /// A hard line break (Shift+Enter)
    HardBreak,
    /// A horizontal rule/divider
    HorizontalRule,
    /// A task/checkbox list
    TaskList,
    /// An item within a taskList
    TaskItem,
}

/// Mark types supported by the Tiptap editor configuration.
///
/// These correspond to the marks from StarterKit in Editor.svelte:
/// bold, italic, strike, code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MarkType {
    /// Bold text
    Bold,
    /// Inline code
    Code,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}
impl From<&str> for TextAlign {
    fn from(value: &str) -> Self {
        match value {
            "center" => Self::Center,
            "right" => Self::Right,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum OrderedListType {
    LowerCaseLetter,
    UpperCaseLetter,
    LowerCaseRoman,
    UpperCaseRoman,
    #[default]
    Number,
}
impl From<&str> for OrderedListType {
    fn from(value: &str) -> Self {
        match value {
            "a" => Self::LowerCaseLetter,
            "A" => Self::UpperCaseLetter,
            "i" => Self::LowerCaseRoman,
            "I" => Self::UpperCaseRoman,
            _ => Self::Number,
        }
    }
}

/// A mark applied to inline content (e.g., bold, italic, link).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mark {
    /// The type of the mark (e.g., Bold, Italic, Code)
    #[serde(rename = "type")]
    pub mark_type: MarkType,
}

/// A Tiptap JSON node or document. Tiptap JSON is the standard format for
/// storing and manipulating Tiptap content. It is equivalent to the JSON
/// representation of a ProseMirror node.
///
/// Tiptap JSON documents are trees of nodes. The root node is usually of type
/// `doc`. Nodes can have other nodes as children. Nodes can also have marks and
/// attributes. Text nodes (nodes with type `text`) have a `text` property and no
/// children.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TipTapNode {
    /// The type of the node (e.g., Doc, Paragraph, Text)
    #[serde(rename = "type")]
    pub node_type: NodeType,
    /// The attributes of the node. Attributes can have any JSON-serializable value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<HashMap<String, serde_json::Value>>,

    /// The children of the node. A node can have other nodes as children.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<TipTapNode>>,

    /// A list of marks of the node. Inline nodes can have marks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marks: Option<Vec<Mark>>,

    /// The text content of the node. This property is only present on text nodes
    /// (i.e. nodes with `type: 'text'`).
    ///
    /// Text nodes cannot have children, but they can have marks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl TipTapNode {
    /// Trace helper: logs what attribute key is being searched and what attrs exist
    fn trace_attr_search(&self, key: &str) {
        match &self.attrs {
            Some(attrs) => {
                let keys: Vec<&str> = attrs.keys().map(|k| k.as_str()).collect();
                log::trace!(
                    "TipTapNode: searching for attr '{}' on node {:?}; available attrs: {:?}",
                    key,
                    self.node_type,
                    keys
                );
            }
            None => {
                log::trace!(
                    "TipTapNode: searching for attr '{}' on node {:?}; no attrs present",
                    key,
                    self.node_type
                );
            }
        }
    }

    /// Trace helper: logs when an attribute has been found
    fn trace_attr_found(&self, key: &str, value: &serde_json::Value) {
        log::trace!(
            "TipTapNode: found attr '{}' = {:?} on node {:?}",
            key,
            value,
            self.node_type
        );
    }

    /// Returns the `language` attribute for `codeBlock` nodes.
    pub fn code_block_language(&self) -> Option<&str> {
        self.trace_attr_search("language");
        if self.node_type != NodeType::CodeBlock {
            return None;
        }
        let v = self.attrs.as_ref()?.get("language")?;
        self.trace_attr_found("language", v);
        v.as_str()
    }

    pub fn is_bold(&self) -> bool {
        if let Some(marks) = &self.marks {
            let found = marks.iter().find(|v| v.mark_type == MarkType::Bold);
            found.is_some()
        } else {
            false
        }
    }

    pub fn text_align(&self) -> Option<TextAlign> {
        self.trace_attr_search("textAlign");
        // Only Paragraph or Heading support textAlign
        if self.node_type != NodeType::Paragraph && self.node_type != NodeType::Heading {
            return None;
        }
        let align = self.attrs.as_ref()?.get("textAlign")?;
        self.trace_attr_found("textAlign", align);
        align.as_str().map(TextAlign::from)
    }

    pub fn heading_level(&self) -> Option<u8> {
        self.trace_attr_search("level");
        let v = self.attrs.as_ref()?.get("level")?;
        self.trace_attr_found("level", v);
        v.as_u64().map(|v| v as u8)
    }

    /// Returns the `start` attribute for `orderedList` nodes.
    pub fn ordered_list_start(&self) -> Option<u64> {
        self.trace_attr_search("start");
        let v = self.attrs.as_ref()?.get("start")?;
        self.trace_attr_found("start", v);
        v.as_u64()
    }

    /// Returns the `type` attribute for `orderedList` nodes (e.g., "1", "a", "A", "i", "I").
    pub fn ordered_list_type(&self) -> Option<OrderedListType> {
        self.trace_attr_search("type");
        let v = self.attrs.as_ref()?.get("type")?;
        self.trace_attr_found("type", v);
        v.as_str().map(OrderedListType::from)
    }

    /// Returns the `checked` attribute for `taskItem` nodes.
    pub fn is_checked(&self) -> Option<bool> {
        self.trace_attr_search("checked");
        let v = self.attrs.as_ref()?.get("checked")?;
        self.trace_attr_found("checked", v);
        v.as_bool()
    }

    pub fn events(&self) -> EventIter<'_> {
        EventIter {
            stack: vec![(self, 0)],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event<'a> {
    NodeStart(&'a TipTapNode),
    NodeEnd(&'a TipTapNode),
    Text(&'a str, &'a TipTapNode),
}

pub struct EventIter<'a> {
    stack: Vec<(&'a TipTapNode, usize)>,
}

impl<'a> Iterator for EventIter<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (node, idx) = self.stack.pop()?;

        if node.node_type == NodeType::Text {
            // Text nodes are leaves: emit Text, then push (node, children+1) so
            // the next pop emits NodeEnd (idx > children len).
            if idx == 0 {
                self.stack.push((node, 1));
                return Some(Event::Text(node.text.as_deref().unwrap_or(""), node));
            }
            // idx > 0 for text nodes — done, no NodeEnd for text
            return self.next();
        }

        let children = node.content.as_deref().unwrap_or(&[]);

        if idx == 0 {
            self.stack.push((node, 1));
            return Some(Event::NodeStart(node));
        }

        let child_idx = idx - 1;
        if child_idx < children.len() {
            self.stack.push((node, idx + 1));
            self.stack.push((&children[child_idx], 0));
            return self.next();
        }

        Some(Event::NodeEnd(node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> TipTapNode {
        serde_json::from_str(include_str!("test.json")).expect("valid tiptap json")
    }

    fn collect_by_type<'a>(node: &'a TipTapNode, ty: NodeType, out: &mut Vec<&'a TipTapNode>) {
        if node.node_type == ty.clone() {
            out.push(node);
        }
        if let Some(children) = &node.content {
            for c in children {
                collect_by_type(c, ty.clone(), out);
            }
        }
    }

    #[test]
    fn paragraph_text_align_variants() {
        let root = fixture();
        let mut paras = Vec::new();
        collect_by_type(&root, NodeType::Paragraph, &mut paras);

        // Right-aligned paragraph exists
        let right = paras
            .iter()
            .find(|n| matches!(n.text_align(), Some(TextAlign::Right)))
            .expect("a right-aligned paragraph");
        assert_eq!(right.text_align(), Some(TextAlign::Right));

        // Center-aligned paragraph exists
        let center = paras
            .iter()
            .find(|n| matches!(n.text_align(), Some(TextAlign::Center)))
            .expect("a center-aligned paragraph");
        assert_eq!(center.text_align(), Some(TextAlign::Center));

        // Paragraph with null/absent textAlign returns None
        let none = paras
            .iter()
            .find(|n| n.attrs.as_ref().and_then(|a| a.get("textAlign")).is_some())
            .and_then(|n| {
                if n.text_align().is_none() {
                    Some(*n)
                } else {
                    None
                }
            })
            .expect("a paragraph with textAlign null that yields None");
        assert!(none.text_align().is_none());
    }

    #[test]
    fn heading_level_and_align() {
        let root = fixture();
        let mut heads = Vec::new();
        collect_by_type(&root, NodeType::Heading, &mut heads);

        // We expect levels 1..=4 to appear at least once
        for expected in [1u8, 2, 3, 4] {
            assert!(
                heads.iter().any(|h| h.heading_level() == Some(expected)),
                "missing heading level {}",
                expected
            );
        }

        // A level-1 centered heading exists
        let h1_center = heads
            .iter()
            .find(|h| h.heading_level() == Some(1) && h.text_align() == Some(TextAlign::Center))
            .expect("a level 1 centered heading");
        assert_eq!(h1_center.heading_level(), Some(1));
        assert_eq!(h1_center.text_align(), Some(TextAlign::Center));
    }

    #[test]
    fn ordered_list_attrs() {
        let root = fixture();
        let mut lists = Vec::new();
        collect_by_type(&root, NodeType::OrderedList, &mut lists);
        let ol = lists.first().expect("an ordered list node");
        assert_eq!(ol.ordered_list_start(), Some(1));
        assert!(ol.ordered_list_type().is_none()); // type: null
    }

    #[test]
    fn task_item_checked_variants() {
        let root = fixture();
        let mut items = Vec::new();
        collect_by_type(&root, NodeType::TaskItem, &mut items);

        assert!(items.iter().any(|n| n.is_checked() == Some(false)));
        assert!(items.iter().any(|n| n.is_checked() == Some(true)));
    }

    #[test]
    fn code_block_language_none_and_non_applicability() {
        let root = fixture();
        let mut blocks = Vec::new();
        collect_by_type(&root, NodeType::CodeBlock, &mut blocks);
        let cb = blocks.first().expect("a code block node");
        assert!(cb.code_block_language().is_none()); // language: null

        // For non-codeBlock nodes, method must also return None
        let mut paras = Vec::new();
        collect_by_type(&root, NodeType::Paragraph, &mut paras);
        let p = paras.first().unwrap();
        assert!(p.code_block_language().is_none());
    }

    #[test]
    fn events_emit_start_text_end() {
        let root = fixture();
        let events: Vec<_> = root.events().collect();

        // First event is NodeStart(doc)
        assert!(
            matches!(events.first(), Some(Event::NodeStart(n)) if n.node_type == NodeType::Doc)
        );
        // Last event is NodeEnd(doc)
        assert!(matches!(events.last(), Some(Event::NodeEnd(n)) if n.node_type == NodeType::Doc));

        // Every NodeStart has a matching NodeEnd
        let starts = events
            .iter()
            .filter(|e| matches!(e, Event::NodeStart(_)))
            .count();
        let ends = events
            .iter()
            .filter(|e| matches!(e, Event::NodeEnd(_)))
            .count();
        assert_eq!(starts, ends);

        // At least one Text event exists
        assert!(events.iter().any(|e| matches!(e, Event::Text(_, _))));
    }

    #[test]
    fn bold_mark_detection() {
        let root = fixture();
        let mut texts = Vec::new();
        collect_by_type(&root, NodeType::Text, &mut texts);

        assert!(texts.iter().any(|t| t.is_bold()));
        assert!(texts.iter().any(|t| !t.is_bold()));
    }
}
