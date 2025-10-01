use consola::{debug, error, info, info_raw, warn};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;

fn bench_basic_logging(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic_logging");

    group.bench_function("info_macro", |b| {
        b.iter(|| {
            info!("Simple info message");
        });
    });

    group.bench_function("info_with_args", |b| {
        b.iter(|| {
            let value = black_box(42);
            info!("Message with value: {}", value);
        });
    });

    group.bench_function("warn_macro", |b| {
        b.iter(|| {
            warn!("Simple warning message");
        });
    });

    group.bench_function("error_macro", |b| {
        b.iter(|| {
            error!("Simple error message");
        });
    });

    group.bench_function("debug_macro", |b| {
        b.iter(|| {
            debug!("Simple debug message");
        });
    });

    group.finish();
}

fn bench_raw_vs_formatted(c: &mut Criterion) {
    let mut group = c.benchmark_group("raw_vs_formatted");

    group.bench_function("formatted_info", |b| {
        b.iter(|| {
            info!("Formatted message");
        });
    });

    group.bench_function("raw_info", |b| {
        b.iter(|| {
            info_raw!("Raw message");
        });
    });

    group.bench_function("formatted_with_args", |b| {
        b.iter(|| {
            let value = black_box(42);
            info!("Value: {}", value);
        });
    });

    group.bench_function("raw_with_args", |b| {
        b.iter(|| {
            let value = black_box(42);
            info_raw!("Value: {}", value);
        });
    });

    group.finish();
}

fn bench_repeated_messages(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeated_messages");

    // Test throttling behavior with different repetition counts
    for count in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(
            BenchmarkId::new("same_message", count),
            count,
            |b, &count| {
                b.iter(|| {
                    for _ in 0..count {
                        info!("Repeated message");
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("unique_messages", count),
            count,
            |b, &count| {
                b.iter(|| {
                    for i in 0..count {
                        info!("Unique message {}", i);
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_format_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_complexity");

    group.bench_function("no_args", |b| {
        b.iter(|| {
            info!("Message with no arguments");
        });
    });

    group.bench_function("one_arg", |b| {
        b.iter(|| {
            let value = black_box(42);
            info!("Message with one argument: {}", value);
        });
    });

    group.bench_function("three_args", |b| {
        b.iter(|| {
            let (a, b, c) = (black_box(1), black_box(2), black_box(3));
            info!("Message with three arguments: {}, {}, {}", a, b, c);
        });
    });

    group.bench_function("five_args", |b| {
        b.iter(|| {
            let values = [
                black_box(1),
                black_box(2),
                black_box(3),
                black_box(4),
                black_box(5),
            ];
            info!(
                "Five args: {}, {}, {}, {}, {}",
                values[0], values[1], values[2], values[3], values[4]
            );
        });
    });

    group.finish();
}

fn bench_string_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_sizes");

    let short = "Short";
    let medium = "This is a medium length message with some detail";
    let long = "This is a very long message that contains a lot of text and could represent a detailed error message or a comprehensive log entry with multiple pieces of information that need to be communicated to the user or system administrator";

    group.bench_function("short_message", |b| {
        b.iter(|| {
            info!("{}", black_box(short));
        });
    });

    group.bench_function("medium_message", |b| {
        b.iter(|| {
            info!("{}", black_box(medium));
        });
    });

    group.bench_function("long_message", |b| {
        b.iter(|| {
            info!("{}", black_box(long));
        });
    });

    group.finish();
}

fn bench_baseline_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("baseline_comparison");

    group.bench_function("println", |b| {
        b.iter(|| {
            println!("{}", black_box("Baseline message"));
        });
    });

    group.bench_function("consola_info", |b| {
        b.iter(|| {
            info!("{}", black_box("Baseline message"));
        });
    });

    group.bench_function("println_with_args", |b| {
        b.iter(|| {
            let value = black_box(42);
            println!("Value: {}", value);
        });
    });

    group.bench_function("consola_info_with_args", |b| {
        b.iter(|| {
            let value = black_box(42);
            info!("Value: {}", value);
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets =
        bench_basic_logging,
        bench_raw_vs_formatted,
        bench_repeated_messages,
        bench_format_complexity,
        bench_string_sizes,
        bench_baseline_comparison
}

criterion_main!(benches);
