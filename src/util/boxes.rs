//! Box drawing utilities.

use crate::util::color::get_color;
use crate::util::string::string_width;

/// A box border style (owned strings to allow colorization).
#[derive(Debug, Clone)]
pub struct BoxBorderStyle {
    /// Top-left corner character.
    pub tl: String,
    /// Top-right corner character.
    pub tr: String,
    /// Bottom-left corner character.
    pub bl: String,
    /// Bottom-right corner character.
    pub br: String,
    /// Horizontal border line character.
    pub h: String,
    /// Vertical border line character.
    pub v: String,
}

fn style_preset(name: &str) -> BoxBorderStyle {
    let s = |c: &str| c.to_string();
    match name {
        "solid" => BoxBorderStyle {
            tl: s("┌"),
            tr: s("┐"),
            bl: s("└"),
            br: s("┘"),
            h: s("─"),
            v: s("│"),
        },
        "double" => BoxBorderStyle {
            tl: s("╔"),
            tr: s("╗"),
            bl: s("╚"),
            br: s("╝"),
            h: s("═"),
            v: s("║"),
        },
        "doubleSingle" | "double_single" => BoxBorderStyle {
            tl: s("╓"),
            tr: s("╖"),
            bl: s("╙"),
            br: s("╜"),
            h: s("─"),
            v: s("║"),
        },
        "doubleSingleRounded" | "double_single_rounded" => BoxBorderStyle {
            tl: s("╭"),
            tr: s("╮"),
            bl: s("╰"),
            br: s("╯"),
            h: s("─"),
            v: s("║"),
        },
        "singleThick" | "single_thick" => BoxBorderStyle {
            tl: s("┏"),
            tr: s("┓"),
            bl: s("┗"),
            br: s("┛"),
            h: s("━"),
            v: s("┃"),
        },
        "singleDouble" | "single_double" => BoxBorderStyle {
            tl: s("╒"),
            tr: s("╕"),
            bl: s("╘"),
            br: s("╛"),
            h: s("═"),
            v: s("│"),
        },
        "singleDoubleRounded" | "single_double_rounded" => BoxBorderStyle {
            tl: s("╭"),
            tr: s("╮"),
            bl: s("╰"),
            br: s("╯"),
            h: s("═"),
            v: s("│"),
        },
        "rounded" => BoxBorderStyle {
            tl: s("╭"),
            tr: s("╮"),
            bl: s("╰"),
            br: s("╯"),
            h: s("─"),
            v: s("│"),
        },
        _ => BoxBorderStyle {
            tl: s("┌"),
            tr: s("┐"),
            bl: s("└"),
            br: s("┘"),
            h: s("─"),
            v: s("│"),
        },
    }
}

/// Box style configuration — border color, preset style, vertical alignment, padding, and margins.
///
/// Created with [`BoxStyle::default()`] or constructed directly. Used by [`BoxOpts`]
/// and consumed by [`box_text`].
#[derive(Debug, Clone)]
pub struct BoxStyle {
    /// Color name for the border (e.g. "red", "blue").
    pub border_color: String,
    /// Border style preset name (e.g. "rounded", "solid", "double").
    pub border_style: String,
    /// Vertical alignment of content ("top", "center", "bottom").
    pub valign: String,
    /// Padding width inside the box around the content.
    pub padding: usize,
    /// Number of spaces to indent the box from the left.
    pub margin_left: usize,
    /// Number of blank lines above the box.
    pub margin_top: usize,
    /// Number of blank lines below the box.
    pub margin_bottom: usize,
}

impl Default for BoxStyle {
    fn default() -> Self {
        Self {
            border_color: "white".into(),
            border_style: "rounded".into(),
            valign: "center".into(),
            padding: 2,
            margin_left: 1,
            margin_top: 1,
            margin_bottom: 1,
        }
    }
}

/// Options for creating a styled box around text.
///
/// Passed to [`box_text`] to control the title and visual style of the box.
#[derive(Debug, Clone, Default)]
pub struct BoxOpts {
    /// Optional title displayed inside the top border.
    pub title: Option<String>,
    /// Optional box style configuration.
    pub style: Option<BoxStyle>,
}

/// Apply `color_fn` to the border portion (after `left_space` margin) of a line.
/// Leaves the line unchanged when `colored` is false, or the line is empty,
/// or it already carries ANSI codes.
fn color_border_line(
    line: &str,
    left_space: &str,
    color_fn: fn(&str) -> String,
    colored: bool,
) -> String {
    if !colored || line.trim().is_empty() {
        return line.to_string();
    }
    if left_space.is_empty() || line.len() <= left_space.len() {
        return color_fn(line);
    }
    let (margin, rest) = line.split_at(left_space.len());
    format!("{}{}", margin, color_fn(rest))
}

/// Draw a styled box around `text` using the given options.
///
/// Supports configurable border style, border color, title, padding, margins,
/// and vertical alignment. The title is centered within the top border line.
/// Returns the fully formatted box as a single string with newlines.
pub fn box_text(text: &str, opts: &BoxOpts) -> String {
    let style = opts.style.clone().unwrap_or_default();
    let preset = style_preset(&style.border_style);
    let color_fn = get_color(&style.border_color);
    let colored = !style.border_color.is_empty() && style.border_color != "white";

    // Colored vertical bar used on content lines (only border part that needs
    // individual coloring, to avoid coloring content text)
    let v = color_fn(&preset.v);

    let text_lines: Vec<&str> = text.split('\n').collect();
    let padding_offset = if style.padding.is_multiple_of(2) {
        style.padding
    } else {
        style.padding + 1
    };

    let max_line_width = text_lines
        .iter()
        .map(|l| string_width(l))
        .max()
        .unwrap_or(0);

    let title_width = opts.title.as_ref().map(|t| string_width(t)).unwrap_or(0);
    let width = max_line_width.max(title_width) + padding_offset;
    let width_offset = width + padding_offset;

    let left_space = if style.margin_left > 0 {
        " ".repeat(style.margin_left)
    } else {
        String::new()
    };

    let mut lines: Vec<String> = Vec::new();

    for _ in 0..style.margin_top {
        lines.push(String::new());
    }

    // Top border — built as plain text then colored once to avoid per-char ANSI breaks
    // between the corner and the horizontal run, which can cause visible seams.
    if let Some(title) = &opts.title {
        let left_count = (width - title_width) / 2;
        let right_count = width - title_width - left_count + padding_offset;
        let raw = format!(
            "{}{}{}{}{}{}",
            left_space,
            preset.tl,
            preset.h.repeat(left_count),
            title,
            preset.h.repeat(right_count),
            preset.tr,
        );
        lines.push(color_border_line(&raw, &left_space, color_fn, colored));
    } else {
        let raw = format!(
            "{}{}{}{}",
            left_space,
            preset.tl,
            preset.h.repeat(width_offset),
            preset.tr,
        );
        lines.push(color_border_line(&raw, &left_space, color_fn, colored));
    }

    let height = text_lines.len() + padding_offset;
    let valign_offset = match style.valign.as_str() {
        "center" => (height - text_lines.len()) / 2,
        "top" => 0,
        _ => height - text_lines.len(),
    };

    for i in 0..height {
        let content = if i < valign_offset || i >= valign_offset + text_lines.len() {
            " ".repeat(width_offset)
        } else {
            let line = text_lines[i - valign_offset];
            let right = " ".repeat(width - string_width(line));
            let left_pad = " ".repeat(padding_offset);
            format!("{}{}{}", left_pad, line, right)
        };
        // All lines use same structure: colored v on both sides, plain content in between.
        // This avoids terminal rendering artifacts from mixing single-span and dual-span lines.
        lines.push(format!("{}{}{}{}", left_space, v, content, v,));
    }

    // Bottom border — single colored span
    let raw = format!(
        "{}{}{}{}",
        left_space,
        preset.bl,
        preset.h.repeat(width_offset),
        preset.br,
    );
    lines.push(color_border_line(&raw, &left_space, color_fn, colored));

    for _ in 0..style.margin_bottom {
        lines.push(String::new());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_style_default() {
        let style = BoxStyle::default();
        assert_eq!(style.border_color, "white");
        assert_eq!(style.border_style, "rounded");
        assert_eq!(style.valign, "center");
        assert_eq!(style.padding, 2);
        assert_eq!(style.margin_left, 1);
        assert_eq!(style.margin_top, 1);
        assert_eq!(style.margin_bottom, 1);
    }

    #[test]
    fn test_box_opts_default() {
        let opts = BoxOpts::default();
        assert!(opts.title.is_none());
        assert!(opts.style.is_none());
    }

    #[test]
    fn test_box_text_default_rounded() {
        let result = box_text("hello", &BoxOpts::default());
        assert!(result.contains("hello"));
        assert!(result.contains('╭'));
        assert!(result.contains('╮'));
        assert!(result.contains('╰'));
        assert!(result.contains('╯'));
        assert!(result.contains('│'));
        assert!(result.contains('─'));
    }

    #[test]
    fn test_box_text_with_title() {
        let opts = BoxOpts {
            title: Some("title".into()),
            ..Default::default()
        };
        let result = box_text("hello", &opts);
        assert!(result.contains("hello"));
        assert!(result.contains("title"));
    }

    #[test]
    fn test_box_text_styles() {
        let styles = [
            "solid",
            "double",
            "rounded",
            "singleThick",
            "doubleSingle",
            "singleDouble",
            "singleDoubleRounded",
            "doubleSingleRounded",
        ];
        for style_name in styles {
            let style = BoxStyle {
                border_style: style_name.into(),
                ..Default::default()
            };
            let opts = BoxOpts {
                style: Some(style),
                ..Default::default()
            };
            let result = box_text("test", &opts);
            assert!(
                result.contains("test"),
                "Style '{}' should contain the text",
                style_name
            );
        }
    }

    #[test]
    fn test_box_text_padding_zero() {
        let style = BoxStyle {
            padding: 0,
            ..Default::default()
        };
        let opts = BoxOpts {
            style: Some(style),
            ..Default::default()
        };
        let result = box_text("hi", &opts);
        assert!(result.contains("hi"));
    }

    #[test]
    fn test_box_text_valign_top() {
        let style = BoxStyle {
            valign: "top".into(),
            ..Default::default()
        };
        let opts = BoxOpts {
            style: Some(style),
            ..Default::default()
        };
        let result = box_text("hi", &opts);
        assert!(result.contains("hi"));
    }

    #[test]
    fn test_box_text_valign_center() {
        let style = BoxStyle {
            valign: "center".into(),
            ..Default::default()
        };
        let opts = BoxOpts {
            style: Some(style),
            ..Default::default()
        };
        let result = box_text("hi", &opts);
        assert!(result.contains("hi"));
    }

    #[test]
    fn test_box_text_valign_bottom() {
        let style = BoxStyle {
            valign: "bottom".into(),
            ..Default::default()
        };
        let opts = BoxOpts {
            style: Some(style),
            ..Default::default()
        };
        let result = box_text("hi", &opts);
        assert!(result.contains("hi"));
    }

    #[test]
    fn test_box_text_colored_border() {
        let style = BoxStyle {
            border_color: "red".into(),
            ..Default::default()
        };
        let opts = BoxOpts {
            style: Some(style),
            ..Default::default()
        };
        // Should not panic, should still produce a box
        let result = box_text("hello", &opts);
        assert!(result.contains("hello"));
        assert!(result.contains('│'));
    }

    #[test]
    fn test_box_text_margins() {
        let style = BoxStyle {
            margin_left: 3,
            margin_top: 2,
            margin_bottom: 2,
            ..Default::default()
        };
        let opts = BoxOpts {
            style: Some(style),
            ..Default::default()
        };
        let result = box_text("hi", &opts);
        assert!(result.contains("hi"));
        // Should have more leading newlines (2 vs 1)
        assert!(
            result.starts_with("\n\n"),
            "Expected two leading newlines, got: {:?}",
            &result[..result.len().min(10)]
        );
    }

    #[test]
    fn test_box_text_multi_line() {
        let result = box_text("line1\nline2\nline3", &BoxOpts::default());
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
        assert!(result.contains("line3"));
    }

    #[test]
    fn test_box_text_empty() {
        let result = box_text("", &BoxOpts::default());
        assert!(!result.is_empty());
        assert!(result.contains('╭'));
        assert!(result.contains('╮'));
        assert!(result.contains('╰'));
        assert!(result.contains('╯'));
    }

    #[test]
    fn test_box_text_style_alternate_names() {
        // doubleSingle and double_single are equivalent
        let style_a = BoxStyle {
            border_style: "doubleSingle".into(),
            ..Default::default()
        };
        let style_b = BoxStyle {
            border_style: "double_single".into(),
            ..Default::default()
        };
        let opts_a = BoxOpts {
            style: Some(style_a),
            ..Default::default()
        };
        let opts_b = BoxOpts {
            style: Some(style_b),
            ..Default::default()
        };
        assert_eq!(box_text("x", &opts_a), box_text("x", &opts_b));
    }
}
