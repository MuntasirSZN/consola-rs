pub fn strip_ansi(input: &str) -> String {
    let bytes = strip_ansi_escapes::strip(input);
    String::from_utf8(bytes).unwrap_or_else(|_| input.to_string())
}

/// Tree formatter with depth and ellipsis support
pub struct TreeFormatter {
    max_depth: usize,
    unicode: bool,
}

impl TreeFormatter {
    pub fn new(max_depth: usize, unicode: bool) -> Self {
        Self { max_depth, unicode }
    }

    pub fn format_lines(&self, lines: &[String], current_depth: usize) -> Vec<String> {
        if current_depth >= self.max_depth && lines.len() > 1 {
            let mut result = lines[..1].to_vec();
            result.push(self.ellipsis_line());
            return result;
        }

        lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let prefix = if i == lines.len() - 1 {
                    self.last_item_prefix()
                } else {
                    self.item_prefix()
                };
                format!("{}{}", prefix, line)
            })
            .collect()
    }

    fn item_prefix(&self) -> &'static str {
        if self.unicode { "├─ " } else { "|- " }
    }

    fn last_item_prefix(&self) -> &'static str {
        if self.unicode { "└─ " } else { "`- " }
    }

    fn ellipsis_line(&self) -> String {
        let prefix = if self.unicode { "└─ " } else { "`- " };
        format!("{}...", prefix)
    }
}

/// Box builder with unicode/ASCII fallback
pub struct BoxBuilder {
    unicode: bool,
    width: Option<usize>,
}

impl BoxBuilder {
    pub fn new(unicode: bool) -> Self {
        Self {
            unicode,
            width: None,
        }
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    pub fn build(&self, title: &str, content: &[String]) -> Vec<String> {
        let width = self.width.unwrap_or_else(|| {
            let title_width = title.chars().count();
            let content_width = content
                .iter()
                .map(|line| line.chars().count())
                .max()
                .unwrap_or(0);
            std::cmp::max(title_width, content_width) + 4
        });

        let mut lines = Vec::new();

        // Top border
        lines.push(self.top_border(title, width));

        // Content lines
        for line in content {
            lines.push(self.content_line(line, width));
        }

        // Bottom border
        lines.push(self.bottom_border(width));

        lines
    }

    fn top_border(&self, title: &str, width: usize) -> String {
        let (tl, tr, h) = if self.unicode {
            ('┌', '┐', '─')
        } else {
            ('+', '+', '-')
        };

        if title.is_empty() {
            format!("{}{}{}", tl, h.to_string().repeat(width - 2), tr)
        } else {
            let title_len = title.chars().count();
            let padding = if width > title_len + 4 {
                width - title_len - 4
            } else {
                0
            };
            let left_pad = padding / 2;
            let right_pad = padding - left_pad;

            format!(
                "{}{} {} {}{}",
                tl,
                h.to_string().repeat(left_pad + 1),
                title,
                h.to_string().repeat(right_pad + 1),
                tr
            )
        }
    }

    fn content_line(&self, content: &str, width: usize) -> String {
        let v = if self.unicode { '│' } else { '|' };
        let content_len = content.chars().count();
        let padding = if width > content_len + 4 {
            width - content_len - 4
        } else {
            0
        };

        format!("{} {} {}{}", v, content, " ".repeat(padding + 1), v)
    }

    fn bottom_border(&self, width: usize) -> String {
        let (bl, br, h) = if self.unicode {
            ('└', '┘', '─')
        } else {
            ('+', '+', '-')
        };

        format!("{}{}{}", bl, h.to_string().repeat(width - 2), br)
    }
}

/// Alignment helpers
pub enum Alignment {
    Left,
    Center,
    Right,
}

pub fn align_text(text: &str, width: usize, alignment: Alignment) -> String {
    let text_len = text.chars().count();
    if text_len >= width {
        return text.to_string();
    }

    let padding = width - text_len;
    match alignment {
        Alignment::Left => format!("{}{}", text, " ".repeat(padding)),
        Alignment::Right => format!("{}{}", " ".repeat(padding), text),
        Alignment::Center => {
            let left_pad = padding / 2;
            let right_pad = padding - left_pad;
            format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
        }
    }
}

/// Error stack parser with cwd and file:// removal
pub fn parse_error_stack(input: &str) -> Vec<String> {
    let current_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    input
        .lines()
        .map(|line| {
            let line = line.trim();
            // Remove file:// prefix if present
            let line = if let Some(stripped) = line.strip_prefix("file://") {
                stripped
            } else {
                line
            };

            // Remove current working directory from absolute paths
            if !current_dir.is_empty() && line.starts_with(&current_dir) {
                let relative = line
                    .strip_prefix(&current_dir)
                    .unwrap_or(line)
                    .trim_start_matches('/');
                if relative.is_empty() {
                    ".".to_string()
                } else {
                    relative.to_string()
                }
            } else {
                line.to_string()
            }
        })
        .collect()
}

/// Stream sinks for output routing
use std::io::{self, Write};

#[cfg(feature = "color")]
use anstyle::{Color, Style};

/// Color/style helper functions wrapping anstyle
#[cfg(feature = "color")]
pub mod style {
    use super::*;

    /// Create a colored text string
    pub fn colored(text: &str, color: Color) -> String {
        let style = Style::new().fg_color(Some(color));
        format!("{}{}{}", style.render(), text, Style::new().render())
    }

    /// Create dim/faded text
    pub fn dim(text: &str) -> String {
        let style = Style::new().dimmed();
        format!("{}{}{}", style.render(), text, Style::new().render())
    }

    /// Create bold text
    pub fn bold(text: &str) -> String {
        let style = Style::new().bold();
        format!("{}{}{}", style.render(), text, Style::new().render())
    }

    /// Predefined colors for log levels
    pub fn info_color() -> Color {
        Color::Ansi(anstyle::AnsiColor::Cyan)
    }

    pub fn success_color() -> Color {
        Color::Ansi(anstyle::AnsiColor::Green)
    }

    pub fn warn_color() -> Color {
        Color::Ansi(anstyle::AnsiColor::Yellow)
    }

    pub fn error_color() -> Color {
        Color::Ansi(anstyle::AnsiColor::Red)
    }

    pub fn debug_color() -> Color {
        Color::Ansi(anstyle::AnsiColor::Magenta)
    }

    pub fn trace_color() -> Color {
        Color::Ansi(anstyle::AnsiColor::Blue)
    }
}

#[cfg(not(feature = "color"))]
pub mod style {
    /// No-op implementations when color feature is disabled
    pub fn colored(text: &str, _color: ()) -> String {
        text.to_string()
    }

    pub fn dim(text: &str) -> String {
        text.to_string()
    }

    pub fn bold(text: &str) -> String {
        text.to_string()
    }
}

pub trait Sink: Send + Sync {
    fn write_all(&self, buf: &[u8]) -> io::Result<()>;
    fn flush(&self) -> io::Result<()>;
}

pub struct StdoutSink;
impl Sink for StdoutSink {
    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        io::stdout().write_all(buf)
    }

    fn flush(&self) -> io::Result<()> {
        io::stdout().flush()
    }
}

pub struct StderrSink;
impl Sink for StderrSink {
    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        io::stderr().write_all(buf)
    }

    fn flush(&self) -> io::Result<()> {
        io::stderr().flush()
    }
}

pub struct TestSink {
    buffer: parking_lot::Mutex<Vec<u8>>,
}

impl TestSink {
    pub fn new() -> Self {
        Self {
            buffer: parking_lot::Mutex::new(Vec::new()),
        }
    }

    pub fn contents(&self) -> String {
        let buffer = self.buffer.lock();
        String::from_utf8_lossy(&buffer).into_owned()
    }

    pub fn clear(&self) {
        self.buffer.lock().clear();
    }
}

impl Default for TestSink {
    fn default() -> Self {
        Self::new()
    }
}

impl Sink for TestSink {
    fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        self.buffer.lock().extend_from_slice(buf);
        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        Ok(())
    }
}
