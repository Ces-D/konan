use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node types supported by the Tiptap editor configuration.
///
/// These correspond to the extensions enabled in Editor.svelte:
/// - StarterKit: doc, paragraph, text, heading, blockquote, bulletList, orderedList,
///   listItem, codeBlock, hardBreak, horizontalRule
/// - TaskList/TaskItem: taskList, taskItem
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NodeType {
    /// The root document node
    Doc,
    /// A paragraph block
    Paragraph,
    /// Inline text content
    Text,
    /// Heading levels 1-6
    Heading,
    /// A blockquote
    Blockquote,
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
    /// Unknown or custom node type
    #[serde(untagged)]
    Other(String),
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
    /// Italic text
    Italic,
    /// Strikethrough text
    Strike,
    /// Inline code
    Code,
    /// Unknown or custom mark type
    #[serde(untagged)]
    Other(String),
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
pub struct JSONContent {
    /// The type of the node (e.g., Doc, Paragraph, Text)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub node_type: Option<NodeType>,
    /// The attributes of the node. Attributes can have any JSON-serializable value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<HashMap<String, serde_json::Value>>,

    /// The children of the node. A node can have other nodes as children.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<JSONContent>>,

    /// A list of marks of the node. Inline nodes can have marks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marks: Option<Vec<Mark>>,

    /// The text content of the node. This property is only present on text nodes
    /// (i.e. nodes with `type: 'text'`).
    ///
    /// Text nodes cannot have children, but they can have marks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Additional arbitrary properties that may be present on the node.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl JSONContent {
    /// Returns the `language` attribute for `codeBlock` nodes.
    pub fn code_block_language(&self) -> Option<&str> {
        if self.node_type != Some(NodeType::CodeBlock) {
            return None;
        }
        self.attrs
            .as_ref()?
            .get("language")?
            .as_str()
    }

    /// Returns the `textAlign` attribute for `paragraph` nodes.
    pub fn paragraph_text_align(&self) -> Option<&str> {
        if self.node_type != Some(NodeType::Paragraph) {
            return None;
        }
        self.attrs
            .as_ref()?
            .get("textAlign")?
            .as_str()
    }

    /// Returns the `level` attribute for `heading` nodes (1-6).
    pub fn heading_level(&self) -> Option<u8> {
        if self.node_type != Some(NodeType::Heading) {
            return None;
        }
        self.attrs
            .as_ref()?
            .get("level")?
            .as_u64()
            .map(|v| v as u8)
    }

    /// Returns the `textAlign` attribute for `heading` nodes.
    pub fn heading_text_align(&self) -> Option<&str> {
        if self.node_type != Some(NodeType::Heading) {
            return None;
        }
        self.attrs
            .as_ref()?
            .get("textAlign")?
            .as_str()
    }

    /// Returns the `start` attribute for `orderedList` nodes.
    pub fn ordered_list_start(&self) -> Option<i64> {
        if self.node_type != Some(NodeType::OrderedList) {
            return None;
        }
        self.attrs
            .as_ref()?
            .get("start")?
            .as_i64()
    }

    /// Returns the `type` attribute for `orderedList` nodes (e.g., "1", "a", "A", "i", "I").
    pub fn ordered_list_type(&self) -> Option<&str> {
        if self.node_type != Some(NodeType::OrderedList) {
            return None;
        }
        self.attrs
            .as_ref()?
            .get("type")?
            .as_str()
    }

    /// Returns the `checked` attribute for `taskItem` nodes.
    pub fn task_item_checked(&self) -> Option<bool> {
        if self.node_type != Some(NodeType::TaskItem) {
            return None;
        }
        self.attrs
            .as_ref()?
            .get("checked")?
            .as_bool()
    }
}

/// A mark applied to inline content (e.g., bold, italic, link).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mark {
    /// The type of the mark (e.g., Bold, Italic, Code)
    #[serde(rename = "type")]
    pub mark_type: MarkType,

    /// The attributes of the mark. Attributes can have any JSON-serializable value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<HashMap<String, serde_json::Value>>,

    /// Additional arbitrary properties that may be present on the mark.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_simple_doc() {
        let json = r#"{
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "Hello "
                        },
                        {
                            "type": "text",
                            "text": "world",
                            "marks": [{ "type": "bold" }]
                        }
                    ]
                }
            ]
        }"#;

        let content: JSONContent = serde_json::from_str(json).unwrap();

        assert_eq!(content.node_type, Some(NodeType::Doc));
        assert!(content.content.is_some());

        let paragraphs = content.content.unwrap();
        assert_eq!(paragraphs.len(), 1);
        assert_eq!(paragraphs[0].node_type, Some(NodeType::Paragraph));

        let text_nodes = paragraphs[0].content.as_ref().unwrap();
        assert_eq!(text_nodes.len(), 2);
        assert_eq!(text_nodes[0].text, Some("Hello ".to_string()));
        assert_eq!(text_nodes[1].text, Some("world".to_string()));

        let marks = text_nodes[1].marks.as_ref().unwrap();
        assert_eq!(marks.len(), 1);
        assert_eq!(marks[0].mark_type, MarkType::Bold);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let content = JSONContent {
            node_type: Some(NodeType::Doc),
            attrs: None,
            content: Some(vec![JSONContent {
                node_type: Some(NodeType::Paragraph),
                attrs: None,
                content: Some(vec![JSONContent {
                    node_type: Some(NodeType::Text),
                    attrs: None,
                    content: None,
                    marks: Some(vec![Mark {
                        mark_type: MarkType::Bold,
                        attrs: None,
                        extra: HashMap::new(),
                    }]),
                    text: Some("Hello".to_string()),
                    extra: HashMap::new(),
                }]),
                marks: None,
                text: None,
                extra: HashMap::new(),
            }]),
            marks: None,
            text: None,
            extra: HashMap::new(),
        };

        let json = serde_json::to_string(&content).unwrap();
        let deserialized: JSONContent = serde_json::from_str(&json).unwrap();

        assert_eq!(content, deserialized);
    }

    #[test]
    fn test_code_block_language() {
        let json = r#"{
            "type": "codeBlock",
            "attrs": { "language": "rust" },
            "content": [{ "type": "text", "text": "let x = 1;" }]
        }"#;
        let node: JSONContent = serde_json::from_str(json).unwrap();
        assert_eq!(node.code_block_language(), Some("rust"));

        // Without language attr
        let json_no_lang = r#"{ "type": "codeBlock" }"#;
        let node_no_lang: JSONContent = serde_json::from_str(json_no_lang).unwrap();
        assert_eq!(node_no_lang.code_block_language(), None);

        // Wrong node type
        let json_para = r#"{ "type": "paragraph", "attrs": { "language": "rust" } }"#;
        let node_para: JSONContent = serde_json::from_str(json_para).unwrap();
        assert_eq!(node_para.code_block_language(), None);
    }

    #[test]
    fn test_paragraph_text_align() {
        let json = r#"{ "type": "paragraph", "attrs": { "textAlign": "center" } }"#;
        let node: JSONContent = serde_json::from_str(json).unwrap();
        assert_eq!(node.paragraph_text_align(), Some("center"));
    }

    #[test]
    fn test_heading_attrs() {
        let json = r#"{ "type": "heading", "attrs": { "level": 2, "textAlign": "right" } }"#;
        let node: JSONContent = serde_json::from_str(json).unwrap();
        assert_eq!(node.heading_level(), Some(2));
        assert_eq!(node.heading_text_align(), Some("right"));
    }

    #[test]
    fn test_ordered_list_attrs() {
        let json = r#"{ "type": "orderedList", "attrs": { "start": 5, "type": "a" } }"#;
        let node: JSONContent = serde_json::from_str(json).unwrap();
        assert_eq!(node.ordered_list_start(), Some(5));
        assert_eq!(node.ordered_list_type(), Some("a"));
    }

    #[test]
    fn test_task_item_checked() {
        let json = r#"{ "type": "taskItem", "attrs": { "checked": true } }"#;
        let node: JSONContent = serde_json::from_str(json).unwrap();
        assert_eq!(node.task_item_checked(), Some(true));

        let json_unchecked = r#"{ "type": "taskItem", "attrs": { "checked": false } }"#;
        let node_unchecked: JSONContent = serde_json::from_str(json_unchecked).unwrap();
        assert_eq!(node_unchecked.task_item_checked(), Some(false));
    }
}
