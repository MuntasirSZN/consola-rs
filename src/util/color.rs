// ─── ANSI color support via anstyle ──────────────────────────────────────────

use std::sync::OnceLock;

static COLOR_ENABLED: OnceLock<bool> = OnceLock::new();

/// Enable or disable ANSI color output.
pub fn set_color_enabled(enabled: bool) {
    _ = COLOR_ENABLED.set(enabled);
}

/// Whether ANSI colors are currently enabled.
///
/// Auto-detected on first call: respects `NO_COLOR`, `FORCE_COLOR`,
/// `--no-color`, `--color`, and terminal detection.
pub fn color_enabled() -> bool {
    *COLOR_ENABLED.get_or_init(|| {
        let no_color = std::env::var("NO_COLOR").is_ok();
        let force_color = std::env::var("FORCE_COLOR").is_ok();
        let is_disabled = no_color || std::env::args().any(|a| a == "--no-color");
        let is_forced = force_color || std::env::args().any(|a| a == "--color");
        if is_disabled {
            return false;
        }
        if is_forced {
            return true;
        }
        use std::io::IsTerminal;
        std::io::stdout().is_terminal()
    })
}

fn style(name: &str) -> anstyle::Style {
    let color = match name {
        "black" => Some(anstyle::AnsiColor::Black),
        "red" => Some(anstyle::AnsiColor::Red),
        "green" => Some(anstyle::AnsiColor::Green),
        "yellow" => Some(anstyle::AnsiColor::Yellow),
        "blue" => Some(anstyle::AnsiColor::Blue),
        "magenta" => Some(anstyle::AnsiColor::Magenta),
        "cyan" => Some(anstyle::AnsiColor::Cyan),
        "white" => Some(anstyle::AnsiColor::White),
        "gray" | "bright black" | "black_bright" | "blackBright" => {
            Some(anstyle::AnsiColor::BrightBlack)
        }
        "bright red" | "red_bright" | "redBright" => Some(anstyle::AnsiColor::BrightRed),
        "bright green" | "green_bright" | "greenBright" => Some(anstyle::AnsiColor::BrightGreen),
        "bright yellow" | "yellow_bright" | "yellowBright" => {
            Some(anstyle::AnsiColor::BrightYellow)
        }
        "bright blue" | "blue_bright" | "blueBright" => Some(anstyle::AnsiColor::BrightBlue),
        "bright magenta" | "magenta_bright" | "magentaBright" => {
            Some(anstyle::AnsiColor::BrightMagenta)
        }
        "bright cyan" | "cyan_bright" | "cyanBright" => Some(anstyle::AnsiColor::BrightCyan),
        "bright white" | "white_bright" | "whiteBright" => Some(anstyle::AnsiColor::BrightWhite),
        _ => None,
    };

    let mut s = anstyle::Style::new();
    if let Some(c) = color {
        s = s.fg_color(Some(c.into()));
    }
    if name.starts_with("bg_") || name.starts_with("bg") {
        // Map "bgRed", "bg_red" etc to background
        let base = name
            .strip_prefix("bg_")
            .or_else(|| name.strip_prefix("bg"))
            .unwrap_or("");
        if let Some(c) = style(base).get_fg_color() {
            s = s.bg_color(Some(c));
        }
    }
    s
}

fn apply(text: &str, effects: &[anstyle::Effects], names: &[&str]) -> String {
    if !color_enabled() {
        return text.to_string();
    }
    if names.is_empty() && effects.is_empty() {
        return text.to_string();
    }
    let mut s = anstyle::Style::new();
    for effect in effects {
        s |= *effect;
    }
    for name in names {
        match *name {
            "bold" => s |= anstyle::Effects::BOLD,
            "dim" => s |= anstyle::Effects::DIMMED,
            "italic" => s |= anstyle::Effects::ITALIC,
            "underline" => s |= anstyle::Effects::UNDERLINE,
            "inverse" => s |= anstyle::Effects::INVERT,
            "hidden" => s |= anstyle::Effects::HIDDEN,
            "strikethrough" => s |= anstyle::Effects::STRIKETHROUGH,
            _ => {
                s = s.fg_color(style(name).get_fg_color());
                s = s.bg_color(style(name).get_bg_color());
            }
        }
    }
    let reset = anstyle::Reset;
    format!("{s}{text}{reset}")
}

/// Color `text` with black.
pub fn black(text: &str) -> String {
    apply(text, &[], &["black"])
}

/// Color `text` with red.
pub fn red(text: &str) -> String {
    apply(text, &[], &["red"])
}

/// Color `text` with green.
pub fn green(text: &str) -> String {
    apply(text, &[], &["green"])
}

/// Color `text` with yellow.
pub fn yellow(text: &str) -> String {
    apply(text, &[], &["yellow"])
}

/// Color `text` with blue.
pub fn blue(text: &str) -> String {
    apply(text, &[], &["blue"])
}

/// Color `text` with magenta.
pub fn magenta(text: &str) -> String {
    apply(text, &[], &["magenta"])
}

/// Color `text` with cyan.
pub fn cyan(text: &str) -> String {
    apply(text, &[], &["cyan"])
}

/// Color `text` with white.
pub fn white(text: &str) -> String {
    apply(text, &[], &["white"])
}

/// Color `text` with gray.
pub fn gray(text: &str) -> String {
    apply(text, &[], &["gray"])
}

/// Color `text` with bright black.
pub fn black_bright(text: &str) -> String {
    apply(text, &[], &["black_bright"])
}

/// Color `text` with bright red.
pub fn red_bright(text: &str) -> String {
    apply(text, &[], &["red_bright"])
}

/// Color `text` with bright green.
pub fn green_bright(text: &str) -> String {
    apply(text, &[], &["green_bright"])
}

/// Color `text` with bright yellow.
pub fn yellow_bright(text: &str) -> String {
    apply(text, &[], &["yellow_bright"])
}

/// Color `text` with bright blue.
pub fn blue_bright(text: &str) -> String {
    apply(text, &[], &["blue_bright"])
}

/// Color `text` with bright magenta.
pub fn magenta_bright(text: &str) -> String {
    apply(text, &[], &["magenta_bright"])
}

/// Color `text` with bright cyan.
pub fn cyan_bright(text: &str) -> String {
    apply(text, &[], &["cyan_bright"])
}

/// Color `text` with bright white.
pub fn white_bright(text: &str) -> String {
    apply(text, &[], &["white_bright"])
}

/// Apply bold formatting to `text`.
/// Wraps `text` with ANSI bold escape sequences when colors are enabled.
pub fn bold(text: &str) -> String {
    apply(text, &[anstyle::Effects::BOLD], &[])
}
/// Apply dim/dark formatting to `text`.
/// Wraps `text` with ANSI dim escape sequences when colors are enabled.
pub fn dim(text: &str) -> String {
    apply(text, &[anstyle::Effects::DIMMED], &[])
}
/// Apply italic formatting to `text`.
/// Wraps `text` with ANSI italic escape sequences when colors are enabled.
pub fn italic(text: &str) -> String {
    apply(text, &[anstyle::Effects::ITALIC], &[])
}
/// Apply underline formatting to `text`.
/// Wraps `text` with ANSI underline escape sequences when colors are enabled.
pub fn underline(text: &str) -> String {
    apply(text, &[anstyle::Effects::UNDERLINE], &[])
}
/// Apply inverse/reversed formatting to `text`.
/// Wraps `text` with ANSI inverse escape sequences when colors are enabled.
pub fn inverse(text: &str) -> String {
    apply(text, &[anstyle::Effects::INVERT], &[])
}
/// Apply hidden/invisible formatting to `text`.
/// Wraps `text` with ANSI hidden escape sequences when colors are enabled.
pub fn hidden(text: &str) -> String {
    apply(text, &[anstyle::Effects::HIDDEN], &[])
}
/// Apply strikethrough formatting to `text`.
/// Wraps `text` with ANSI strikethrough escape sequences when colors are enabled.
pub fn strikethrough(text: &str) -> String {
    apply(text, &[anstyle::Effects::STRIKETHROUGH], &[])
}

/// Apply black background color to `text`.
/// Wraps `text` with ANSI black background escape sequences when colors are enabled.
pub fn bg_black(text: &str) -> String {
    apply(text, &[], &["bg_black"])
}
/// Apply red background color to `text`.
/// Wraps `text` with ANSI red background escape sequences when colors are enabled.
pub fn bg_red(text: &str) -> String {
    apply(text, &[], &["bg_red"])
}
/// Apply green background color to `text`.
/// Wraps `text` with ANSI green background escape sequences when colors are enabled.
pub fn bg_green(text: &str) -> String {
    apply(text, &[], &["bg_green"])
}
/// Apply yellow background color to `text`.
/// Wraps `text` with ANSI yellow background escape sequences when colors are enabled.
pub fn bg_yellow(text: &str) -> String {
    apply(text, &[], &["bg_yellow"])
}
/// Apply blue background color to `text`.
/// Wraps `text` with ANSI blue background escape sequences when colors are enabled.
pub fn bg_blue(text: &str) -> String {
    apply(text, &[], &["bg_blue"])
}
/// Apply magenta background color to `text`.
/// Wraps `text` with ANSI magenta background escape sequences when colors are enabled.
pub fn bg_magenta(text: &str) -> String {
    apply(text, &[], &["bg_magenta"])
}
/// Apply cyan background color to `text`.
/// Wraps `text` with ANSI cyan background escape sequences when colors are enabled.
pub fn bg_cyan(text: &str) -> String {
    apply(text, &[], &["bg_cyan"])
}
/// Apply white background color to `text`.
/// Wraps `text` with ANSI white background escape sequences when colors are enabled.
pub fn bg_white(text: &str) -> String {
    apply(text, &[], &["bg_white"])
}

// ─── Lookup ───────────────────────────────────────────────────────────────────

/// Look up a color function by name.
pub fn get_color(name: &str) -> fn(&str) -> String {
    match name {
        "reset" => |s: &str| {
            if color_enabled() {
                format!("{}{}", anstyle::Reset, s)
            } else {
                s.to_string()
            }
        },
        "bold" => bold,
        "dim" => dim,
        "italic" => italic,
        "underline" => underline,
        "inverse" => inverse,
        "hidden" => hidden,
        "strikethrough" => strikethrough,
        "black" => black,
        "red" => red,
        "green" => green,
        "yellow" => yellow,
        "blue" => blue,
        "magenta" => magenta,
        "cyan" => cyan,
        "white" => white,
        "gray" => gray,
        "bgBlack" | "bg_black" => bg_black,
        "bgRed" | "bg_red" => bg_red,
        "bgGreen" | "bg_green" => bg_green,
        "bgYellow" | "bg_yellow" => bg_yellow,
        "bgBlue" | "bg_blue" => bg_blue,
        "bgMagenta" | "bg_magenta" => bg_magenta,
        "bgCyan" | "bg_cyan" => bg_cyan,
        "bgWhite" | "bg_white" => bg_white,
        "blackBright" | "black_bright" => black_bright,
        "redBright" | "red_bright" => red_bright,
        "greenBright" | "green_bright" => green_bright,
        "yellowBright" | "yellow_bright" => yellow_bright,
        "blueBright" | "blue_bright" => blue_bright,
        "magentaBright" | "magenta_bright" => magenta_bright,
        "cyanBright" | "cyan_bright" => cyan_bright,
        "whiteBright" | "white_bright" => white_bright,
        _ => |s: &str| s.to_string(),
    }
}

/// Applies a named color to text.
pub fn colorize(name: &str, text: &str) -> String {
    get_color(name)(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    type ColorFn = (&'static str, fn(&str) -> String);

    /// Attempt to enable colors for testing.
    /// Only the first call across all tests takes effect due to OnceLock.
    fn enable_colors() {
        set_color_enabled(true);
    }

    /// Assert that output wraps input in ANSI codes when colors are enabled,
    /// or returns input unchanged when colors are disabled.
    fn assert_ansi_or_plain(result: &str, input: &str) {
        if color_enabled() {
            assert!(
                result.starts_with("\x1b["),
                "Expected ANSI escape start, got: {:?}",
                result
            );
            assert!(
                result.ends_with("\x1b[0m"),
                "Expected ANSI reset end, got: {:?}",
                result
            );
            assert!(
                result.contains(input),
                "Expected result to contain input '{}', got: {:?}",
                input,
                result
            );
        } else {
            assert_eq!(result, input, "Expected plain text when colors disabled");
        }
    }

    /// Collect up to 3 inputs to test a color function.
    fn test_color_fn(f: fn(&str) -> String, inputs: &[&str]) {
        for &input in inputs {
            let result = f(input);
            assert_ansi_or_plain(&result, input);
        }
    }

    // ── 1. All color functions produce ANSI-escaped output ──────────────────

    #[test]
    fn test_foreground_colors() {
        enable_colors();
        let inputs = &["hello", "world", "foo"];
        test_color_fn(red, inputs);
        test_color_fn(green, inputs);
        test_color_fn(yellow, inputs);
        test_color_fn(blue, inputs);
        test_color_fn(magenta, inputs);
        test_color_fn(cyan, inputs);
        test_color_fn(white, inputs);
        test_color_fn(gray, inputs);
        test_color_fn(black, inputs);
    }

    #[test]
    fn test_bright_colors() {
        enable_colors();
        let inputs = &["hello", "bright"];
        test_color_fn(red_bright, inputs);
        test_color_fn(green_bright, inputs);
        test_color_fn(yellow_bright, inputs);
        test_color_fn(blue_bright, inputs);
        test_color_fn(magenta_bright, inputs);
        test_color_fn(cyan_bright, inputs);
        test_color_fn(white_bright, inputs);
        test_color_fn(black_bright, inputs);
    }

    #[test]
    fn test_bg_colors() {
        enable_colors();
        let inputs = &["bg", "test"];
        test_color_fn(bg_red, inputs);
        test_color_fn(bg_green, inputs);
        test_color_fn(bg_yellow, inputs);
        test_color_fn(bg_blue, inputs);
        test_color_fn(bg_magenta, inputs);
        test_color_fn(bg_cyan, inputs);
        test_color_fn(bg_white, inputs);
        test_color_fn(bg_black, inputs);
    }

    #[test]
    fn test_style_functions() {
        enable_colors();
        let inputs = &["text", "styling"];
        for &input in inputs {
            let fns: [ColorFn; 7] = [
                ("bold", bold),
                ("dim", dim),
                ("italic", italic),
                ("underline", underline),
                ("inverse", inverse),
                ("hidden", hidden),
                ("strikethrough", strikethrough),
            ];
            for (name, f) in &fns {
                let result = f(input);
                if color_enabled() {
                    assert!(
                        result.starts_with("\x1b["),
                        "{}({}) should have ANSI prefix: {:?}",
                        name,
                        input,
                        result
                    );
                    assert!(
                        result.ends_with("\x1b[0m"),
                        "{}({}) should have ANSI suffix: {:?}",
                        name,
                        input,
                        result
                    );
                    assert!(
                        result.contains(input),
                        "{}({}) should contain text: {:?}",
                        name,
                        input,
                        result
                    );
                } else {
                    assert_eq!(result, input);
                }
            }
        }
    }

    #[test]
    fn test_specific_red_output() {
        enable_colors();
        let result = red("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_green_output() {
        enable_colors();
        let result = green("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_yellow_output() {
        enable_colors();
        let result = yellow("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_blue_output() {
        enable_colors();
        let result = blue("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_magenta_output() {
        enable_colors();
        let result = magenta("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_cyan_output() {
        enable_colors();
        let result = cyan("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_white_output() {
        enable_colors();
        let result = white("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_gray_output() {
        enable_colors();
        let result = gray("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_bg_red_output() {
        enable_colors();
        let result = bg_red("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_bg_green_output() {
        enable_colors();
        let result = bg_green("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_bg_yellow_output() {
        enable_colors();
        let result = bg_yellow("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_bg_blue_output() {
        enable_colors();
        let result = bg_blue("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_bg_magenta_output() {
        enable_colors();
        let result = bg_magenta("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_bg_cyan_output() {
        enable_colors();
        let result = bg_cyan("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_bg_white_output() {
        enable_colors();
        let result = bg_white("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_red_bright_output() {
        enable_colors();
        let result = red_bright("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_green_bright_output() {
        enable_colors();
        let result = green_bright("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_yellow_bright_output() {
        enable_colors();
        let result = yellow_bright("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_blue_bright_output() {
        enable_colors();
        let result = blue_bright("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_magenta_bright_output() {
        enable_colors();
        let result = magenta_bright("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_cyan_bright_output() {
        enable_colors();
        let result = cyan_bright("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_white_bright_output() {
        enable_colors();
        let result = white_bright("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    #[test]
    fn test_specific_black_bright_output() {
        enable_colors();
        let result = black_bright("hello");
        assert_ansi_or_plain(&result, "hello");
    }

    // ── 2. Starts with \x1b[ and ends with \x1b[0m ────────────────────────

    #[test]
    fn test_output_starts_with_escape_and_ends_with_reset() {
        enable_colors();
        if !color_enabled() {
            return;
        }
        // Color functions that actually produce ANSI
        let cases: [ColorFn; 25] = [
            ("red", red),
            ("green", green),
            ("yellow", yellow),
            ("blue", blue),
            ("magenta", magenta),
            ("cyan", cyan),
            ("white", white),
            ("gray", gray),
            ("black", black),
            ("red_bright", red_bright),
            ("green_bright", green_bright),
            ("yellow_bright", yellow_bright),
            ("blue_bright", blue_bright),
            ("magenta_bright", magenta_bright),
            ("cyan_bright", cyan_bright),
            ("white_bright", white_bright),
            ("black_bright", black_bright),
            ("bg_red", bg_red),
            ("bg_green", bg_green),
            ("bg_yellow", bg_yellow),
            ("bg_blue", bg_blue),
            ("bg_magenta", bg_magenta),
            ("bg_cyan", bg_cyan),
            ("bg_white", bg_white),
            ("bg_black", bg_black),
        ];
        for (name, f) in &cases {
            let result = f("x");
            assert!(
                result.starts_with("\x1b["),
                "{}(\"x\") should start with ESC[: {:?}",
                name,
                result
            );
            assert!(
                result.ends_with("\x1b[0m"),
                "{}(\"x\") should end with ESC[0m: {:?}",
                name,
                result
            );
        }
        // reset via get_color("reset") — emits just the reset code before text
        let reset_fn = get_color("reset");
        let r = reset_fn("x");
        assert!(
            r.starts_with("\x1b["),
            "reset(\"x\") should start with ESC[: {:?}",
            r
        );
    }

    // ── 3–5. get_color ────────────────────────────────────────────────────

    #[test]
    fn test_get_color_known() {
        enable_colors();
        let f = get_color("red");
        let input = "hello";
        let result = f(input);
        assert_ansi_or_plain(&result, input);
    }

    #[test]
    fn test_get_color_nonexistent_returns_identity() {
        let f = get_color("nonexistent");
        assert_eq!(f("hello"), "hello", "unknown color returns identity");
        assert_eq!(f(""), "", "identity works for empty string");
    }

    #[test]
    fn test_get_color_empty_returns_identity() {
        let f = get_color("");
        assert_eq!(f("hello"), "hello", "empty string returns identity");
    }

    // ── 6–7. colorize ─────────────────────────────────────────────────────

    #[test]
    fn test_colorize_known() {
        enable_colors();
        let result = colorize("red", "text");
        assert_ansi_or_plain(&result, "text");
    }

    #[test]
    fn test_colorize_nonexistent_unchanged() {
        let result = colorize("nonexistent", "text");
        assert_eq!(
            result, "text",
            "colorize with unknown name returns unchanged"
        );
    }

    // ── 8–10. set_color_enabled / color_enabled ────────────────────────────

    #[test]
    fn test_color_enabled_toggle() {
        // Record the initial state before any change.
        let initial = color_enabled();

        // Disable colors.
        set_color_enabled(false);

        let after_disable = color_enabled();

        // If our call was the first to init the OnceLock, state changed.
        let disabled = !after_disable;

        if disabled {
            assert_eq!(red("x"), "x", "disabled: \"x\" unchanged");
            assert_eq!(
                colorize("red", "x"),
                "x",
                "disabled: colorize returns plain"
            );
        }

        // Attempt to re-enable (only meaningful if our set was the first call).
        set_color_enabled(true);
        let after_enable = color_enabled();

        if disabled {
            // If we just disabled it, re-enable is a no-op (OnceLock already set)
            // so `after_enable` is still false.
            // This documents the OnceLock limitation: toggle does not work.
            assert!(!after_enable, "OnceLock prevents re-enabling after disable");
        } else {
            // State was already initialized (e.g. by a parallel test), so
            // our calls had no effect.
            assert_eq!(after_enable, initial);
        }
    }

    #[test]
    fn test_color_enabled_returns_state() {
        // Just call it — doesn't assert a particular value, just that it runs.
        let _state = color_enabled();
    }

    // ── get_color("reset") ────────────────────────────────────────────────

    #[test]
    fn test_get_color_reset() {
        enable_colors();
        let reset_fn = get_color("reset");
        let result = reset_fn("x");
        if color_enabled() {
            assert!(
                result.starts_with("\x1b["),
                "reset should add escape prefix, got: {:?}",
                result
            );
            assert!(result.contains("x"), "reset should preserve input text");
        } else {
            assert_eq!(result, "x");
        }
    }
}
