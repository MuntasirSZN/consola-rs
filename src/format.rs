use crate::error_chain::format_chain_lines;
use crate::record::LogRecord;
use std::env;

#[derive(Debug, Clone)]
pub struct FormatOptions {
    pub date: bool,
    pub colors: bool,
    pub compact: bool,
    pub columns: Option<usize>,
    pub error_level: usize,
    pub unicode: bool,
    pub show_tag: bool,
    pub show_type: bool,
    pub show_repetition: bool,
    pub show_stack: bool,
    pub show_additional: bool,
    pub show_meta: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            date: true,
            colors: true,
            compact: false,
            columns: None,
            error_level: 16,
            unicode: true,
            show_tag: true,
            show_type: true,
            show_repetition: true,
            show_stack: false,
            show_additional: true,
            show_meta: true,
        }
    }
}

impl FormatOptions {
    pub fn adaptive() -> Self {
        let mut o = Self::default();
        // Env overrides
        if env::var("NO_COLOR").is_ok() {
            o.colors = false;
        }
        if let Ok(force) = env::var("FORCE_COLOR") {
            if !force.is_empty() && force != "0" {
                o.colors = true;
            }
        }
        if let Ok(compact) = env::var("CONSOLA_COMPACT") {
            if compact == "1" {
                o.compact = true;
            }
        }
        // Terminal width detect
        o.columns = detect_terminal_width();
        o
    }
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub text: String,
    pub style: Option<SegmentStyle>,
}

#[derive(Debug, Clone)]
pub struct SegmentStyle {
    pub fg_color: Option<String>,
    pub bg_color: Option<String>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
}

pub fn build_basic_segments(record: &LogRecord, opts: &FormatOptions) -> Vec<Segment> {
    let mut v = Vec::new();

    if opts.date {
        let ts = {
            let z = jiff::Zoned::now();
            z.to_string()
        };

        v.push(Segment {
            text: ts,
            style: Some(SegmentStyle {
                fg_color: Some("gray".into()),
                bg_color: None,
                bold: false,
                dim: true,
                italic: false,
                underline: false,
            }),
        });
    }

    if opts.show_type {
        v.push(Segment {
            text: format!("[{}]", record.type_name),
            style: Some(SegmentStyle {
                fg_color: Some("cyan".into()),
                bg_color: None,
                bold: true,
                dim: false,
                italic: false,
                underline: false,
            }),
        });
    }

    if opts.show_tag {
        if let Some(tag) = &record.tag {
            v.push(Segment {
                text: format!("[{tag}]"),
                style: Some(SegmentStyle {
                    fg_color: Some("magenta".into()),
                    bg_color: None,
                    bold: false,
                    dim: false,
                    italic: true,
                    underline: false,
                }),
            });
        }
    }

    if let Some(msg) = &record.message {
        v.push(Segment {
            text: msg.clone(),
            style: None,
        });
    }

    if opts.show_repetition && record.repetition_count > 1 {
        v.push(Segment {
            text: format!(" (x{})", record.repetition_count),
            style: Some(SegmentStyle {
                fg_color: Some("gray".into()),
                bg_color: None,
                bold: false,
                dim: true,
                italic: false,
                underline: false,
            }),
        });
    }

    // Additional args (if any) appended as JSON-ish list (placeholder formatting)
    if opts.show_additional {
        if let Some(additional) = &record.additional {
            if !additional.is_empty() {
                let mut out = String::new();
                out.push(' ');
                out.push('[');
                for (i, a) in additional.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&a.to_string());
                }
                out.push(']');
                v.push(Segment {
                    text: out,
                    style: Some(SegmentStyle {
                        fg_color: Some("cyan".into()),
                        bg_color: None,
                        bold: false,
                        dim: true,
                        italic: false,
                        underline: false,
                    }),
                });
            }
        }
    }

    // Meta key=value pairs
    if opts.show_meta {
        if let Some(meta) = &record.meta {
            if !meta.is_empty() {
                let mut out = String::new();
                out.push(' ');
                out.push('{');
                for (i, (k, vval)) in meta.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(k);
                    out.push('=');
                    out.push_str(&vval.to_string());
                }
                out.push('}');
                v.push(Segment {
                    text: out,
                    style: Some(SegmentStyle {
                        fg_color: Some("yellow".into()),
                        bg_color: None,
                        bold: false,
                        dim: true,
                        italic: false,
                        underline: false,
                    }),
                });
            }
        }
    }

    // Stack (multi-line) appended if enabled
    if opts.show_stack {
        if let Some(stack) = &record.stack {
            if !stack.is_empty() {
                for (i, line) in stack.iter().enumerate() {
                    let prefix = if i == 0 { "\n" } else { "" };
                    v.push(Segment {
                        text: format!("{prefix}{line}"),
                        style: Some(SegmentStyle {
                            fg_color: Some("gray".into()),
                            bg_color: None,
                            bold: false,
                            dim: false,
                            italic: false,
                            underline: false,
                        }),
                    });
                }
            }
        } else if let Some(crate::record::ArgValue::Error(msg)) = record
            .args
            .iter()
            .find(|a| matches!(a, crate::record::ArgValue::Error(_)))
        {
            // Build pseudo chain from single error message only (placeholder)
            let lines = format_chain_lines(&[msg.clone()], opts.error_level);
            for (i, line) in lines.iter().enumerate() {
                let prefix = if i == 0 { "\n" } else { "" };
                v.push(Segment {
                    text: format!("{prefix}{line}"),
                    style: Some(SegmentStyle {
                        fg_color: Some("gray".into()),
                        bg_color: None,
                        bold: false,
                        dim: false,
                        italic: false,
                        underline: false,
                    }),
                });
            }
        }
    }

    v
}

/// Attempt to detect terminal column width.
pub fn detect_terminal_width() -> Option<usize> {
    if let Ok(cols) = env::var("COLUMNS") {
        if let Ok(n) = cols.parse::<usize>() {
            if n > 0 {
                return Some(n);
            }
        }
    }
    None
}

/// Compute printable width of concatenated segments (simplistic; excludes ANSI codes)
pub fn compute_line_width(segments: &[Segment]) -> usize {
    segments.iter().map(|s| display_width(&s.text)).sum()
}

fn display_width(s: &str) -> usize {
    #[cfg(feature = "fancy")]
    {
        use unicode_width::UnicodeWidthStr;
        UnicodeWidthStr::width(s)
    }
    #[cfg(not(feature = "fancy"))]
    {
        s.chars().count()
    }
}
