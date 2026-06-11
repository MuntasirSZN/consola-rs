use std::sync::Arc;

use consola::{
    BoxOpts, Consola, ConsolaOpts, LogObject, LogObjectInput, LogType, TreeItem, TreeOptions,
    box_text, format_tree,
    reporters::{BasicReporter, FancyReporter},
    set_color_enabled, string_width, strip_ansi,
    types::{LogContext, Reporter},
    util::color::{bold, green, red},
};
use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn make_log_obj(ty: LogType, args: &[&str], tag: &str) -> LogObject {
    LogObject {
        level: ty.level(),
        r#type: ty,
        tag: tag.to_string(),
        message: None,
        additional: None,
        args: args.iter().map(|s| s.to_string()).collect(),
        timestamp_ms: 0,
        title: None,
        badge: false,
        icon: None,
        style: None,
        error: None,
    }
}

fn make_log_ctx() -> LogContext {
    LogContext {
        options: Arc::new(ConsolaOpts::default()),
    }
}

// ─── 1. LogObjectInput + LogObject construction ───────────────────────────────

#[divan::bench]
fn log_object_building(bencher: divan::Bencher) {
    let input = LogObjectInput::new()
        .message("hello world")
        .tag("test-tag")
        .type_(LogType::Info)
        .title("my-title")
        .additional("extra context line");

    bencher.bench_local(|| {
        let ty = input.r#type.unwrap_or(LogType::Log);
        let mut obj = LogObject::new(ty);
        obj.level = ty.level();
        obj.tag
            .clone_from(input.tag.as_ref().unwrap_or(&String::new()));
        obj.message = input.message.clone();
        obj.additional = input.additional.clone();
        obj.title = input.title.clone();
        obj.args.clone_from(&input.args);
        divan::black_box(obj);
    });
}

// ─── 2. BasicReporter formatting ──────────────────────────────────────────────

#[divan::bench]
fn basic_reporter_format(bencher: divan::Bencher) {
    let reporter = BasicReporter;
    let log_obj = make_log_obj(LogType::Info, &["hello", "world"], "tag");
    let ctx = make_log_ctx();

    bencher.bench_local(|| {
        reporter
            .format(divan::black_box(&log_obj), divan::black_box(&ctx))
            .unwrap()
    });
}

// ─── 3. FancyReporter formatting ──────────────────────────────────────────────

#[divan::bench]
fn fancy_reporter_format(bencher: divan::Bencher) {
    set_color_enabled(true);
    let reporter = FancyReporter::new();
    let log_obj = make_log_obj(LogType::Info, &["hello", "world"], "tag");
    let ctx = make_log_ctx();

    bencher.bench_local(|| {
        reporter
            .format(divan::black_box(&log_obj), divan::black_box(&ctx))
            .unwrap()
    });
}

// ─── 4. Color functions (red, green, bold) ────────────────────────────────────

#[divan::bench]
fn color_red(bencher: divan::Bencher) {
    set_color_enabled(true);
    let text = "hello world";
    bencher.bench_local(|| divan::black_box(red(divan::black_box(text))));
}

#[divan::bench]
fn color_green(bencher: divan::Bencher) {
    set_color_enabled(true);
    let text = "hello world";
    bencher.bench_local(|| divan::black_box(green(divan::black_box(text))));
}

#[divan::bench]
fn color_bold(bencher: divan::Bencher) {
    set_color_enabled(true);
    let text = "hello world";
    bencher.bench_local(|| divan::black_box(bold(divan::black_box(text))));
}

// ─── 5. string_width, strip_ansi ──────────────────────────────────────────────

#[divan::bench]
fn strip_ansi_bench() -> String {
    strip_ansi("\x1b[31mhello world\x1b[0m")
}

#[divan::bench]
fn string_width_plain() -> usize {
    string_width("hello world")
}

#[divan::bench]
fn string_width_emoji() -> usize {
    string_width("héllo wörld ☀️")
}

// ─── 6. box_text ──────────────────────────────────────────────────────────────

#[divan::bench]
fn box_text_default() -> String {
    box_text("Hello, World!", &BoxOpts::default())
}

#[divan::bench]
fn box_text_with_title() -> String {
    let opts = BoxOpts {
        title: Some("Benchmark".to_string()),
        ..BoxOpts::default()
    };
    box_text("Hello, World!", &opts)
}

// ─── 7. format_tree ───────────────────────────────────────────────────────────

#[divan::bench]
fn format_tree_bench(bencher: divan::Bencher) {
    let items = vec![TreeItem::Node {
        text: "root".into(),
        children: vec![
            TreeItem::Text("child1".into()),
            TreeItem::Text("child2".into()),
            TreeItem::Node {
                text: "subtree".into(),
                children: vec![
                    TreeItem::Text("leaf1".into()),
                    TreeItem::Text("leaf2".into()),
                ],
                color: None,
            },
        ],
        color: None,
    }];
    let opts = TreeOptions::default();

    bencher.bench_local(|| divan::black_box(format_tree(divan::black_box(&items), &opts)));
}

// ─── 8. Consola::info with different message sizes ────────────────────────────

#[divan::bench]
fn consola_info_small(bencher: divan::Bencher) {
    let consola = Consola::new(ConsolaOpts {
        level: 3,
        reporters: vec![Box::new(BasicReporter)],
        ..ConsolaOpts::default()
    });

    bencher.bench_local(|| consola.info(divan::black_box("Hello, World!")));
}

#[divan::bench]
fn consola_info_medium(bencher: divan::Bencher) {
    let consola = Consola::new(ConsolaOpts {
        level: 3,
        reporters: vec![Box::new(BasicReporter)],
        ..ConsolaOpts::default()
    });

    bencher.bench_local(|| {
        consola.info(divan::black_box(
            "This is a medium-length message for benchmarking purposes.",
        ))
    });
}

#[divan::bench]
fn consola_info_large(bencher: divan::Bencher) {
    let consola = Consola::new(ConsolaOpts {
        level: 3,
        reporters: vec![Box::new(BasicReporter)],
        ..ConsolaOpts::default()
    });
    let large =
        "This is a long message that stresses the formatting pipeline of the logger. ".repeat(20);

    bencher.bench_local(|| consola.info(divan::black_box(&large)));
}

#[divan::bench]
fn consola_info_multiline(bencher: divan::Bencher) {
    let consola = Consola::new(ConsolaOpts {
        level: 3,
        reporters: vec![Box::new(BasicReporter)],
        ..ConsolaOpts::default()
    });

    bencher
        .bench_local(|| consola.info(divan::black_box("Line 1\nLine 2\nLine 3\nLine 4\nLine 5")));
}
