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
                // Stack or error chain
                if opts.show_stack {
                    if let Some(stack) = &record.stack {
                        if !stack.is_empty() {
                            for (i, line) in stack.iter().enumerate() {
                                let prefix = if i == 0 { "\n" } else { "" };
                                // Enhanced stack line coloring: gray "at", cyan path
                                let styled_line = if line.trim().starts_with("at ") {
                                    let parts: Vec<&str> = line.splitn(2, "at ").collect();
                                    if parts.len() == 2 {
                                        format!("{}at {}", parts[0], parts[1])
                                    } else {
                                        line.clone()
                                    }
                                } else {
                                    line.clone()
                                };

                                v.push(Segment {
                                    text: format!("{prefix}{styled_line}"),
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
                        }
                    } else if let Some(chain) = &record.error_chain {
                        let limited: Vec<String> =
                            chain.iter().take(opts.error_level).cloned().collect();
                        for (i, line) in limited.iter().enumerate() {
                            let prefix = if i == 0 { "\n" } else { "" };

                            // Multi-line message normalization with indentation
                            let text = if i == 0 {
                                normalize_multiline_message(line, "")
                            } else {
                                let caused_by_msg = format!("Caused by: {line}");
                                normalize_multiline_message(&caused_by_msg, "           ") // Indent continuation lines
                            };

                            v.push(Segment {
                                text: format!("{prefix}{text}"),
                                style: Some(SegmentStyle {
                                    fg_color: Some("red".into()),
                                    bg_color: None,
                                    bold: false,
                                    dim: false,
                                    italic: false,
                                    underline: false,
                                }),
                            });
                        }
                        if chain.len() > opts.error_level {
                            v.push(Segment {
                                text: format!(
                                    "\n(+{} more causes)",
                                    chain.len() - opts.error_level
                                ),
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
                    }
                }
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

    v
}

/// Attempt to detect terminal column width.
/// Tries in order:
/// 1. COLUMNS environment variable
/// 2. Terminal size detection (Unix only)
/// 3. Falls back to None (let caller decide default)
pub fn detect_terminal_width() -> Option<usize> {
    // Try COLUMNS env var first
    if let Ok(cols) = env::var("COLUMNS") {
        if let Ok(n) = cols.parse::<usize>() {
            if n > 0 {
                return Some(n);
            }
        }
    }

    // Try terminal size detection on Unix
    #[cfg(unix)]
    {
        if let Some(size) = get_terminal_size_unix() {
            return Some(size);
        }
    }

    // Windows terminal detection could be added here with winapi
    #[cfg(windows)]
    {
        // TODO: Windows console API for terminal width
    }

    None
}

#[cfg(unix)]
fn get_terminal_size_unix() -> Option<usize> {
    use std::io::{self, IsTerminal};

    // Check if stdout is a terminal
    if !io::stdout().is_terminal() {
        return None;
    }

    // Use ioctl to get terminal size
    // SAFETY: This is safe because we're just querying terminal size
    // and not modifying anything
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            return Some(ws.ws_col as usize);
        }
    }

    None
}

/// Normalize multi-line error messages with proper indentation
fn normalize_multiline_message(message: &str, indent: &str) -> String {
    let lines: Vec<&str> = message.lines().collect();
    if lines.len() <= 1 {
        return message.to_string();
    }

    let mut result = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            result.push('\n');
            result.push_str(indent);
        }
        result.push_str(line);
    }
    result
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
