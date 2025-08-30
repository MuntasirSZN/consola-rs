use crate::record::LogRecord;

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
        }
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
    
    v
}
