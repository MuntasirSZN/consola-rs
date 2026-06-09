//! Utility modules providing string formatting, ANSI color support, box drawing,
//! tree display formatting, and log object detection helpers.

/// Box drawing utilities for creating styled text boxes.
pub mod boxes;
/// ANSI color and styling functions.
pub mod color;
/// Log object detection helpers.
pub mod log;
/// Unicode-aware string utilities (alignment, ANSI stripping).
pub mod string;
/// Tree structure display formatting.
pub mod tree;

pub use boxes::{BoxOpts, BoxStyle, box_text};
pub use color::{color_enabled, colorize, get_color, set_color_enabled};
pub use string::{align, center_align, left_align, right_align, string_width, strip_ansi};
pub use tree::{TreeItem, TreeOptions, format_tree};
