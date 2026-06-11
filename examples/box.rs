//! ─── box.rs ──────────────────────────────────────────────────────────────────
//! Demonstrates `box_text` and `format_tree` utilities with various styles
//! — matching the consola-js `box.ts` and `tree.ts` examples.

use consola::{
    BoxOpts, BoxStyle, TreeItem, TreeOptions, box_text, create_fancy_consola, format_tree,
    log_levels,
};

fn main() {
    let consola = create_fancy_consola(Some(log_levels::VERBOSE));

    // ── Box: default rounded ─────────────────────────────────────────────────
    consola.box_("I am the default banner");

    // ── Box: with title via log_obj ──────────────────────────────────────────
    consola.log_obj(
        &consola::LogObjectInput::new()
            .type_(consola::LogType::Box)
            .message("short msg")
            .title("longer title"),
    );

    // ── Box: styled with padding and colored border ──────────────────────────
    println!(
        "{}",
        box_text(
            "I am a banner with different options",
            &BoxOpts {
                title: Some("Box with options".into()),
                style: Some(BoxStyle {
                    padding: 1,
                    border_color: "magenta".into(),
                    border_style: "double-single-rounded".into(),
                    ..BoxStyle::default()
                }),
            },
        ),
    );

    // ── Box: update notification style ───────────────────────────────────────
    println!(
        "{}",
        box_text(
            "`v1.0.2` → `v2.0.0`\n\nRun `cargo install consola` to update",
            &BoxOpts {
                title: Some("Update available for `consola`".into()),
                style: Some(BoxStyle {
                    padding: 2,
                    border_color: "yellow".into(),
                    border_style: "rounded".into(),
                    ..BoxStyle::default()
                }),
            },
        ),
    );

    // ── Tree: flat keyword list ──────────────────────────────────────────────
    let keyword_strings = [
        "console",
        "logger",
        "reporter",
        "elegant",
        "cli",
        "universal",
        "unified",
        "prompt",
        "clack",
        "format",
        "error",
        "stacktrace",
    ];
    let keywords: Vec<TreeItem> = keyword_strings
        .iter()
        .map(|s| TreeItem::Text(s.to_string()))
        .collect();
    println!("{}", format_tree(&keywords, &TreeOptions::default()));

    // ── Tree: with custom prefix ─────────────────────────────────────────────
    println!(
        "{}",
        format_tree(
            &keywords,
            &TreeOptions {
                color: Some("cyan".into()),
                prefix: "  |  ".into(),
                ..TreeOptions::default()
            },
        ),
    );

    // ── Tree: deep hierarchy ─────────────────────────────────────────────────
    let deep_tree = format_tree(
        &[
            TreeItem::Node {
                text: "format".into(),
                color: Some("red".into()),
                children: vec![],
            },
            TreeItem::Node {
                text: "consola".into(),
                color: Some("yellow".into()),
                children: vec![
                    TreeItem::Node {
                        text: "logger".into(),
                        color: Some("green".into()),
                        children: vec![
                            TreeItem::Node {
                                text: "reporter".into(),
                                color: Some("cyan".into()),
                                children: vec![],
                            },
                            TreeItem::Node {
                                text: "test".into(),
                                color: Some("magenta".into()),
                                children: vec![TreeItem::Text("nice tree".into())],
                            },
                        ],
                    },
                    TreeItem::Node {
                        text: "reporter".into(),
                        color: Some("bold".into()),
                        children: vec![],
                    },
                    TreeItem::Text("test".into()),
                ],
            },
        ],
        &TreeOptions::default(),
    );
    println!("{}", deep_tree);
    println!();

    // ── Tree: max depth ──────────────────────────────────────────────────────
    println!(
        "{}",
        format_tree(
            &[
                TreeItem::Node {
                    text: "format".into(),
                    color: Some("red".into()),
                    children: vec![],
                },
                TreeItem::Node {
                    text: "consola".into(),
                    color: Some("yellow".into()),
                    children: vec![
                        TreeItem::Node {
                            text: "logger".into(),
                            color: Some("green".into()),
                            children: vec![
                                TreeItem::Node {
                                    text: "reporter".into(),
                                    color: Some("cyan".into()),
                                    children: vec![],
                                },
                                TreeItem::Node {
                                    text: "test".into(),
                                    color: Some("magenta".into()),
                                    children: vec![TreeItem::Text("nice tree".into())],
                                },
                            ],
                        },
                        TreeItem::Node {
                            text: "reporter".into(),
                            color: Some("bold".into()),
                            children: vec![],
                        },
                        TreeItem::Text("test".into()),
                    ],
                },
            ],
            &TreeOptions {
                max_depth: Some(2),
                ellipsis: "---".into(),
                ..TreeOptions::default()
            },
        )
    );
}
