// ─── String utilities: strip ANSI, alignment ──────────────────────────────────

/// Strip ANSI escape codes from a string.
pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == 0x5B {
            // ANSI sequence: ESC [
            i += 2;
            // Skip parameter bytes
            while i < bytes.len() && bytes[i] >= 0x20 && bytes[i] <= 0x3F {
                i += 1;
            }
            // Skip intermediate bytes
            while i < bytes.len() && bytes[i] >= 0x20 && bytes[i] <= 0x2F {
                i += 1;
            }
            // Final byte (0x40-0x7E) or skip if not present
            if i < bytes.len() && bytes[i] >= 0x40 && bytes[i] <= 0x7E {
                i += 1;
            }
        } else if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == 0x5D {
            // OSC sequence: ESC ] ... BEL or ST
            i += 2;
            while i < bytes.len()
                && bytes[i] != 0x07
                && !(bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == 0x5C)
            {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == 0x07 {
                i += 1;
            } else if i + 1 < bytes.len() {
                i += 2;
            }
        } else {
            // Decode the full UTF-8 character (not just one byte)
            let c = text[i..]
                .chars()
                .next()
                .unwrap_or(std::char::REPLACEMENT_CHARACTER);
            out.push(c);
            i += c.len_utf8();
        }
    }
    out
}

/// Visible width of a string, accounting for wide Unicode characters
/// (e.g. CJK, emoji, em dashes) and zero-width characters.
pub fn string_width(text: &str) -> usize {
    use unicode_width::UnicodeWidthStr;
    strip_ansi(text).as_str().width()
}

/// Center-align a string within `len` columns.
pub fn center_align(str: &str, len: usize, space: &str) -> String {
    let width = string_width(str);
    if width >= len {
        return str.to_string();
    }
    let free = len - width;
    let free_left = free / 2;
    let sp = space.chars().next().unwrap_or(' ');
    let mut out = String::with_capacity(len);
    for i in 0..len {
        if i < free_left || i >= free_left + width {
            out.push(sp);
        } else {
            out.push(str.chars().nth(i - free_left).unwrap_or(sp));
        }
    }
    out
}

/// Right-align a string within `len` columns.
pub fn right_align(str: &str, len: usize, space: &str) -> String {
    let width = string_width(str);
    if width >= len {
        return str.to_string();
    }
    let free = len - width;
    let sp = space.chars().next().unwrap_or(' ');
    let mut out = String::with_capacity(len);
    for i in 0..len {
        out.push(if i < free {
            sp
        } else {
            str.chars().nth(i - free).unwrap_or(sp)
        });
    }
    out
}

/// Left-align a string within `len` columns.
pub fn left_align(str: &str, len: usize, space: &str) -> String {
    let width = string_width(str);
    if width >= len {
        return str.to_string();
    }
    let sp = space.chars().next().unwrap_or(' ');
    let mut out = String::with_capacity(len);
    let chars: Vec<char> = str.chars().collect();
    for i in 0..len {
        out.push(if i < chars.len() { chars[i] } else { sp });
    }
    out
}

/// Align a string (left/right/center).
pub fn align(alignment: &str, str: &str, len: usize, space: &str) -> String {
    match alignment {
        "left" => left_align(str, len, space),
        "right" => right_align(str, len, space),
        "center" => center_align(str, len, space),
        _ => str.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_sgr() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn test_strip_ansi_osc() {
        assert_eq!(strip_ansi("\x1b]0;title\x07content"), "content");
    }

    #[test]
    fn test_strip_ansi_osc_st() {
        // OSC terminated by ST (ESC \\)
        assert_eq!(strip_ansi("\x1b]0;title\x1b\\content"), "content");
    }

    #[test]
    fn test_strip_ansi_plain() {
        assert_eq!(strip_ansi("hello world"), "hello world");
    }

    #[test]
    fn test_strip_ansi_empty() {
        assert_eq!(strip_ansi(""), "");
    }

    #[test]
    fn test_string_width_plain() {
        assert_eq!(string_width("hello"), 5);
    }

    #[test]
    fn test_string_width_ansi() {
        assert_eq!(string_width("\x1b[31mhello\x1b[0m"), 5);
    }

    #[test]
    fn test_string_width_cjk() {
        assert_eq!(string_width("你好"), 4);
    }

    #[test]
    fn test_string_width_empty() {
        assert_eq!(string_width(""), 0);
    }

    #[test]
    fn test_center_align_even() {
        assert_eq!(center_align("hi", 6, " "), "  hi  ");
    }

    #[test]
    fn test_center_align_odd() {
        // width=2, len=5, free=3, free_left=1 => " hi  "
        assert_eq!(center_align("hi", 5, " "), " hi  ");
    }

    #[test]
    fn test_center_align_exact() {
        assert_eq!(center_align("hi", 2, " "), "hi");
    }

    #[test]
    fn test_center_align_too_long() {
        assert_eq!(center_align("hello", 3, " "), "hello");
    }

    #[test]
    fn test_right_align_shorter() {
        assert_eq!(right_align("hi", 5, " "), "   hi");
    }

    #[test]
    fn test_right_align_exact() {
        assert_eq!(right_align("hi", 2, " "), "hi");
    }

    #[test]
    fn test_right_align_longer() {
        assert_eq!(right_align("hello world", 5, " "), "hello world");
    }

    #[test]
    fn test_left_align_shorter() {
        assert_eq!(left_align("hi", 5, " "), "hi   ");
    }

    #[test]
    fn test_left_align_exact() {
        assert_eq!(left_align("hi", 2, " "), "hi");
    }

    #[test]
    fn test_left_align_longer() {
        assert_eq!(left_align("hello world", 5, " "), "hello world");
    }

    #[test]
    fn test_align_center() {
        assert_eq!(align("center", "hi", 6, " "), "  hi  ");
    }

    #[test]
    fn test_align_left() {
        assert_eq!(align("left", "hi", 5, " "), "hi   ");
    }

    #[test]
    fn test_align_right() {
        assert_eq!(align("right", "hi", 5, " "), "   hi");
    }

    #[test]
    fn test_align_unknown() {
        assert_eq!(align("unknown", "hi", 5, " "), "hi");
    }
}
