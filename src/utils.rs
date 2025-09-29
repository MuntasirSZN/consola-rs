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
