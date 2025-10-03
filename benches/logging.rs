use consola::{debug, error, info, info_raw, success, warn};

fn main() {
    divan::main();
}

#[divan::bench]
fn info_macro() {
    info!("Simple info message");
}

#[divan::bench]
fn info_with_args() {
    let value = divan::black_box(42);
    info!("Message with value: {}", value);
}

#[divan::bench]
fn warn_macro() {
    warn!("Simple warning message");
}

#[divan::bench]
fn error_macro() {
    error!("Simple error message");
}

#[divan::bench]
fn debug_macro() {
    debug!("Simple debug message");
}

#[divan::bench]
fn success_macro() {
    success!("Simple success message");
}

#[divan::bench(name = "formatted_info")]
fn formatted_info() {
    info!("Formatted message");
}

#[divan::bench(name = "raw_info")]
fn raw_info() {
    info_raw!("Raw message");
}

#[divan::bench(name = "formatted_with_args")]
fn formatted_with_args() {
    let value = divan::black_box(42);
    info!("Value: {}", value);
}

#[divan::bench(name = "raw_with_args")]
fn raw_with_args() {
    let value = divan::black_box(42);
    info_raw!("Value: {}", value);
}

#[divan::bench(args = [10, 100, 1000])]
fn repeated_same_message(n: usize) {
    for _ in 0..n {
        info!("Repeated message");
    }
}

#[divan::bench(args = [10, 100, 1000])]
fn repeated_unique_messages(n: usize) {
    for i in 0..n {
        info!("Unique message {}", i);
    }
}

#[divan::bench]
fn format_no_args() {
    info!("Message with no arguments");
}

#[divan::bench]
fn format_one_arg() {
    let value = divan::black_box(42);
    info!("Message with one argument: {}", value);
}

#[divan::bench]
fn format_three_args() {
    let (a, b, c) = (
        divan::black_box(1),
        divan::black_box(2),
        divan::black_box(3),
    );
    info!("Message with three arguments: {}, {}, {}", a, b, c);
}

#[divan::bench]
fn format_five_args() {
    let values = [
        divan::black_box(1),
        divan::black_box(2),
        divan::black_box(3),
        divan::black_box(4),
        divan::black_box(5),
    ];
    info!(
        "Five args: {}, {}, {}, {}, {}",
        values[0], values[1], values[2], values[3], values[4]
    );
}

#[divan::bench]
fn string_short() {
    let msg = divan::black_box("Short");
    info!("{}", msg);
}

#[divan::bench]
fn string_medium() {
    let msg = divan::black_box("This is a medium length message with some detail");
    info!("{}", msg);
}

#[divan::bench]
fn string_long() {
    let msg = divan::black_box(
        "This is a very long message that contains a lot of text and could represent a detailed error message or a comprehensive log entry with multiple pieces of information that need to be communicated to the user or system administrator",
    );
    info!("{}", msg);
}

#[divan::bench]
fn baseline_println() {
    println!("{}", divan::black_box("Baseline message"));
}

#[divan::bench]
fn baseline_consola_info() {
    info!("{}", divan::black_box("Baseline message"));
}

#[divan::bench]
fn baseline_println_with_args() {
    let value = divan::black_box(42);
    println!("Value: {}", value);
}

#[divan::bench]
fn baseline_consola_info_with_args() {
    let value = divan::black_box(42);
    info!("Value: {}", value);
}
