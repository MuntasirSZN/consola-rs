//! Utility functions for formatting tree structures.

use crate::util::color::colorize;

/// A tree item - either a string or an object with text, children, and color.
#[derive(Debug, Clone)]
pub enum TreeItem {
    /// A leaf node containing only text.
    Text(String),
    /// A node with text, child items, and an optional color.
    Node {
        /// Display text for this node.
        text: String,
        /// Child tree items.
        children: Vec<TreeItem>,
        /// Optional color name for the node text.
        color: Option<String>,
    },
}

impl From<&str> for TreeItem {
    fn from(s: &str) -> Self {
        TreeItem::Text(s.to_string())
    }
}

impl From<String> for TreeItem {
    fn from(s: String) -> Self {
        TreeItem::Text(s)
    }
}

/// Options for tree formatting.
#[derive(Debug, Clone)]
pub struct TreeOptions {
    /// Optional color name to apply to the entire tree.
    pub color: Option<String>,
    /// Indentation prefix for each tree level.
    pub prefix: String,
    /// Maximum depth to render; beyond this, `ellipsis` is shown.
    pub max_depth: Option<usize>,
    /// Text shown when a subtree is truncated by `max_depth`.
    pub ellipsis: String,
}

impl Default for TreeOptions {
    fn default() -> Self {
        Self {
            color: None,
            prefix: "  ".into(),
            max_depth: None,
            ellipsis: "...".into(),
        }
    }
}

/// Format items into a tree string.
pub fn format_tree(items: &[TreeItem], options: &TreeOptions) -> String {
    let tree = build_tree(items, options);
    if let Some(color) = &options.color {
        colorize(color, &tree)
    } else {
        tree
    }
}

fn build_tree(items: &[TreeItem], options: &TreeOptions) -> String {
    let mut out = String::new();
    let total = items.len().saturating_sub(1);
    for (i, item) in items.iter().enumerate() {
        let is_limit = options.max_depth == Some(0);
        if is_limit {
            let ellipsis = format!("{}{}\n", options.prefix, options.ellipsis);
            let line = match item {
                TreeItem::Text(_) => ellipsis,
                TreeItem::Node { color: Some(c), .. } => colorize(c, &ellipsis),
                _ => ellipsis,
            };
            out.push_str(&line);
            return out;
        }

        let is_last = i == total;
        let prefix = if is_last {
            format!("{}└─", options.prefix)
        } else {
            format!("{}├─", options.prefix)
        };

        match item {
            TreeItem::Text(text) => {
                out.push_str(&format!("{}{}\n", prefix, text));
            }
            TreeItem::Node {
                text,
                children,
                color,
            } => {
                let log = format!("{}{}\n", prefix, text);
                if let Some(c) = color {
                    out.push_str(&colorize(c, &log));
                } else {
                    out.push_str(&log);
                }

                if !children.is_empty() {
                    let child_prefix = if is_last {
                        format!("{}  ", options.prefix)
                    } else {
                        format!("{}│ ", options.prefix)
                    };
                    let child_opts = TreeOptions {
                        prefix: child_prefix,
                        max_depth: options.max_depth.map(|d| d.saturating_sub(1)),
                        ..options.clone()
                    };
                    out.push_str(&build_tree(children, &child_opts));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_options_default() {
        let opts = TreeOptions::default();
        assert_eq!(opts.color, None);
        assert_eq!(opts.prefix, "  ");
        assert_eq!(opts.max_depth, None);
        assert_eq!(opts.ellipsis, "...");
    }

    #[test]
    fn test_tree_item_text() {
        let item = TreeItem::Text("leaf".into());
        let result = format_tree(&[item], &TreeOptions::default());
        assert_eq!(result, "  └─leaf\n");
    }

    #[test]
    fn test_tree_item_text_from_str() {
        let item: TreeItem = "leaf".into();
        let result = format_tree(&[item], &TreeOptions::default());
        assert_eq!(result, "  └─leaf\n");
    }

    #[test]
    fn test_tree_item_text_from_string() {
        let item: TreeItem = String::from("leaf").into();
        let result = format_tree(&[item], &TreeOptions::default());
        assert_eq!(result, "  └─leaf\n");
    }

    #[test]
    fn test_tree_item_node() {
        let items = vec![TreeItem::Node {
            text: "root".into(),
            children: vec![TreeItem::Text("a".into()), TreeItem::Text("b".into())],
            color: None,
        }];
        let result = format_tree(&items, &TreeOptions::default());
        assert!(result.contains("root"));
        assert!(result.contains("a"));
        assert!(result.contains("b"));
        // Root is the only item (is_last=true), so child_prefix = "    " (no │).
        // Children "a" (not last) uses ├─, "b" (last) uses └─.
        assert!(!result.contains("│"));
        assert!(result.contains("├─"));
        assert!(result.contains("└─"));
    }

    #[test]
    fn test_format_tree_basic() {
        let items = vec![TreeItem::Text("a".into()), TreeItem::Text("b".into())];
        let result = format_tree(&items, &TreeOptions::default());
        assert_eq!(result, "  ├─a\n  └─b\n");
    }

    #[test]
    fn test_format_tree_single() {
        let items = vec![TreeItem::Text("only".into())];
        let result = format_tree(&items, &TreeOptions::default());
        assert_eq!(result, "  └─only\n");
    }

    #[test]
    fn test_format_tree_max_depth() {
        let items = vec![TreeItem::Node {
            text: "root".into(),
            children: vec![TreeItem::Text("hidden".into())],
            color: None,
        }];
        let opts = TreeOptions {
            max_depth: Some(1),
            ..Default::default()
        };
        let result = format_tree(&items, &opts);
        assert!(result.contains("root"));
        assert!(result.contains("..."));
        assert!(!result.contains("hidden"));
    }

    #[test]
    fn test_format_tree_max_depth_zero() {
        let items = vec![TreeItem::Node {
            text: "root".into(),
            children: vec![TreeItem::Text("child".into())],
            color: None,
        }];
        let opts = TreeOptions {
            max_depth: Some(0),
            ..Default::default()
        };
        let result = format_tree(&items, &opts);
        assert!(result.contains("..."));
        assert!(!result.contains("root"));
    }

    #[test]
    fn test_format_tree_custom_prefix() {
        let items = vec![TreeItem::Text("a".into()), TreeItem::Text("b".into())];
        let opts = TreeOptions {
            prefix: "  * ".into(),
            ..Default::default()
        };
        let result = format_tree(&items, &opts);
        assert_eq!(result, "  * ├─a\n  * └─b\n");
    }

    #[test]
    fn test_format_tree_with_color() {
        let items = vec![TreeItem::Text("hello".into())];
        let opts = TreeOptions {
            color: Some("red".into()),
            ..Default::default()
        };
        let result = format_tree(&items, &opts);
        // Text must appear in output regardless of color environment
        assert!(result.contains("hello"));
        assert!(result.contains("└─"));
    }

    #[test]
    fn test_format_tree_node_with_color() {
        let items = vec![TreeItem::Node {
            text: "colored".into(),
            children: vec![],
            color: Some("blue".into()),
        }];
        let result = format_tree(&items, &TreeOptions::default());
        // Must contain the node text
        assert!(result.contains("colored"));
    }

    #[test]
    fn test_format_tree_nested() {
        // Two siblings at root level, each with children — this creates │ connectors
        let items = vec![
            TreeItem::Node {
                text: "A".into(),
                children: vec![TreeItem::Text("a1".into())],
                color: None,
            },
            TreeItem::Node {
                text: "B".into(),
                children: vec![TreeItem::Text("b1".into())],
                color: None,
            },
        ];
        let result = format_tree(&items, &TreeOptions::default());
        assert!(result.contains("A"));
        assert!(result.contains("a1"));
        assert!(result.contains("B"));
        assert!(result.contains("b1"));
        // A (not last) has child_prefix="  │  " so its child has a │ connector.
        // B (last) has child_prefix="     " so its child does not.
        assert!(result.contains("│"));
        assert!(result.contains("├─"));
        assert!(result.contains("└─"));
    }

    #[test]
    fn test_format_tree_empty() {
        let result = format_tree(&[], &TreeOptions::default());
        assert_eq!(result, "");
    }
}
