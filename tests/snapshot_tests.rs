use consola::*;
use strip_ansi_escapes::strip;

fn render_fancy_line(type_name: &str, msg: &str) -> (String, String) {
    let reporter = FancyReporter {
        opts: FormatOptions {
            date: false,
            ..FormatOptions::default()
        },
    };
    let record = LogRecord::new(type_name, None, vec![msg.into()]);
    // enable stack / error chain display explicitly for error types if any chain present
    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let colored = String::from_utf8(buf).unwrap();
    let plain_bytes = strip(colored.as_bytes());
    let plain = String::from_utf8(plain_bytes).unwrap();
    (colored, plain)
}

#[test]
fn snapshot_fancy_info_basic() {
    let (colored, plain) = render_fancy_line("info", "hello world");
    insta::assert_snapshot!("fancy_info_basic_colored", colored);
    insta::assert_snapshot!("fancy_info_basic_plain", plain);
}

#[test]
fn snapshot_fancy_error_chain_limited() {
    use anyhow::anyhow;
    let reporter = FancyReporter {
        opts: FormatOptions {
            date: false,
            show_stack: true,
            error_level: 2,
            ..FormatOptions::default()
        },
    };
    let err = anyhow!("root cause")
        .context("middle layer")
        .context("top context");
    let err_ref: &(dyn std::error::Error + 'static) = err.as_ref();
    let record =
        LogRecord::new("error", None, vec!["processing failed".into()]).attach_dyn_error(err_ref);
    let mut buf: Vec<u8> = Vec::new();
    reporter.emit(&record, &mut buf).unwrap();
    let colored = String::from_utf8(buf).unwrap();
    let plain_bytes = strip(colored.as_bytes());
    let plain = String::from_utf8(plain_bytes).unwrap();
    insta::assert_snapshot!("fancy_error_chain_colored", colored);
    insta::assert_snapshot!("fancy_error_chain_plain", plain);
}
