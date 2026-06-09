//! ─── formats.rs ──────────────────────────────────────────────────────────────
//! Demonstrates utility formatting functions: `box_text`, `format_tree`, and
//! colour / style helpers.

use consola::{
    BoxOpts, BoxStyle, TreeItem, TreeOptions, box_text, format_tree,
    util::color::{bold, dim, green, italic, red, underline, yellow},
};

fn main() {
    // ── box_text with different styles ────────────────────────────────────────

    let msg = "consola-rs: Elegant Console Logger";

    // Default (rounded, white border)
    println!("{}", box_text(msg, &BoxOpts::default()));

    // Double-line border with green colour
    println!(
        "{}",
        box_text(
            "Double border — green",
            &BoxOpts {
                style: Some(BoxStyle {
                    border_color: "green".into(),
                    border_style: "double".into(),
                    ..BoxStyle::default()
                }),
                ..BoxOpts::default()
            },
        ),
    );

    // Solid border with red colour and a title
    println!(
        "{}",
        box_text(
            "Solid border with title",
            &BoxOpts {
                title: Some(" Alert ".into()),
                style: Some(BoxStyle {
                    border_color: "red".into(),
                    border_style: "solid".into(),
                    ..BoxStyle::default()
                }),
            },
        ),
    );

    // Rounded border with blue colour, extra padding
    println!(
        "{}",
        box_text(
            "Padded content\nsecond line",
            &BoxOpts {
                style: Some(BoxStyle {
                    border_color: "blue".into(),
                    border_style: "rounded".into(),
                    padding: 4,
                    ..BoxStyle::default()
                }),
                ..BoxOpts::default()
            },
        ),
    );

    // ── format_tree ───────────────────────────────────────────────────────────

    let tree = format_tree(
        &[
            TreeItem::Node {
                text: "src".into(),
                color: Some("yellow".into()),
                children: vec![
                    TreeItem::Node {
                        text: "main.rs".into(),
                        color: None,
                        children: vec![],
                    },
                    TreeItem::Node {
                        text: "lib.rs".into(),
                        color: None,
                        children: vec![],
                    },
                    TreeItem::Node {
                        text: "utils".into(),
                        color: Some("cyan".into()),
                        children: vec![
                            TreeItem::Text("mod.rs".into()),
                            TreeItem::Text("string.rs".into()),
                            TreeItem::Text("color.rs".into()),
                        ],
                    },
                ],
            },
            TreeItem::Node {
                text: "tests".into(),
                color: Some("green".into()),
                children: vec![TreeItem::Text("integration.rs".into())],
            },
            TreeItem::Text("Cargo.toml".into()),
        ],
        &TreeOptions {
            color: None,
            ..TreeOptions::default()
        },
    );
    println!("{}", tree);

    // ── Color functions ───────────────────────────────────────────────────────

    println!("{}", red("This text is red"));
    println!("{}", green("This text is green"));
    println!("{}", yellow("This text is yellow"));
    println!("{}", bold("This text is bold"));
    println!("{}", dim("This text is dim"));
    println!("{}", italic("This text is italic"));
    println!("{}", underline("This text is underlined"));
    println!("{}", bold(&green("Bold and green")));
    println!("{}", red(&italic("Red and italic")));
    println!("{}", yellow(&underline("Yellow and underlined")));
    println!("{}", dim(&red("Dim red text")));
}
