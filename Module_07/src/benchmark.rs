use lib::{parallel_quicksort, sequential_quicksort};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

fn pseudo_rand(seed: u64) -> i64 {
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    h.finish() as i64
}

#[derive(Clone, Copy, Debug)]
enum InputKind {
    Random,
    Sorted,
    Reverse,
    Duplicate,
}

impl InputKind {
    fn label(self) -> &'static str {
        match self {
            InputKind::Random => "random",
            InputKind::Sorted => "sorted",
            InputKind::Reverse => "reverse",
            InputKind::Duplicate => "duplicate",
        }
    }
}

fn generate(kind: InputKind, n: usize) -> Vec<i64> {
    match kind {
        InputKind::Random => (0..n as u64).map(pseudo_rand).collect(),
        InputKind::Sorted => (0..n as i64).collect(),
        InputKind::Reverse => (0..n as i64).rev().collect(),
        InputKind::Duplicate => (0..n as i64).map(|i| i % 100).collect(),
    }
}

/// Run `f` exactly `reps` times and return the median duration.
fn median_time<F: Fn()>(reps: usize, f: F) -> Duration {
    let mut times: Vec<Duration> = (0..reps)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed()
        })
        .collect();
    times.sort();
    times[reps / 2]
}

fn fmt_ms(d: Duration) -> String {
    format!("{:.1}", d.as_secs_f64() * 1000.0)
}

fn main() {
    let sizes: &[usize] = &[10_000, 100_000, 1_000_000, 5_000_000];
    let kinds = [
        InputKind::Random,
        InputKind::Sorted,
        InputKind::Reverse,
        InputKind::Duplicate,
    ];
    let cutoffs: &[usize] = &[256, 1_024, 4_096, 16_384];

    // Fewer reps for huge inputs to keep runtime reasonable
    let reps_for = |n: usize| if n >= 1_000_000 { 3 } else { 5 };

    // ── Part A: Sequential vs Parallel (best cutoff = 4096) ──────────────
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  TABLE 1 — Sequential vs Parallel (cutoff = 4096)               ║");
    println!("╠════════════╦══════════════╦═══════════╦══════════╦═══════════════╣");
    println!("║ Input Type ║ Size         ║ Seq (ms)  ║ Par (ms) ║ Speedup       ║");
    println!("╠════════════╬══════════════╬═══════════╬══════════╬═══════════════╣");

    for &kind in &kinds {
        for &n in sizes {
            let reps = reps_for(n);
            let base = generate(kind, n);

            let seq_time = median_time(reps, || {
                let mut d = base.clone();
                sequential_quicksort(&mut d);
            });

            let par_time = median_time(reps, || {
                let mut d = base.clone();
                parallel_quicksort(&mut d, 4_096);
            });

            let speedup = seq_time.as_secs_f64() / par_time.as_secs_f64();

            println!(
                "║ {:<10} ║ {:>12} ║ {:>9} ║ {:>8} ║ {:>12.2}x ║",
                kind.label(),
                format_size(n),
                fmt_ms(seq_time),
                fmt_ms(par_time),
                speedup
            );
        }
        println!("╠════════════╬══════════════╬═══════════╬══════════╬═══════════════╣");
    }
    println!("╚════════════╩══════════════╩═══════════╩══════════╩═══════════════╝");

    // ── Part B: Cutoff Analysis (random, 1 000 000) ──────────────────────
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  TABLE 2 — Cutoff Analysis  (random, n = 1 000 000)         ║");
    println!("╠════════════╦═════════════╦════════════╦═════════════════════╣");
    println!("║ Cutoff     ║ Par (ms)    ║ Seq (ms)   ║ Speedup             ║");
    println!("╠════════════╬═════════════╬════════════╬═════════════════════╣");

    let n = 1_000_000;
    let reps = 3;
    let base = generate(InputKind::Random, n);

    let seq_time = median_time(reps, || {
        let mut d = base.clone();
        sequential_quicksort(&mut d);
    });

    for &cutoff in cutoffs {
        let par_time = median_time(reps, || {
            let mut d = base.clone();
            parallel_quicksort(&mut d, cutoff);
        });
        let speedup = seq_time.as_secs_f64() / par_time.as_secs_f64();
        println!(
            "║ {:>10} ║ {:>11} ║ {:>10} ║ {:>18.2}x ║",
            cutoff,
            fmt_ms(par_time),
            fmt_ms(seq_time),
            speedup
        );
    }
    println!("╚════════════╩═════════════╩════════════╩═════════════════════╝");

    // ── Part C: Full Cutoff × Size × Kind Matrix ─────────────────────────
    println!("\n╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║  TABLE 3 — Parallel speedup matrix  (speedup vs sequential)             ║");
    println!("╠══════════════╦══════════╦════════╦════════╦════════╦════════╦════════════╣");
    print!("║ {:12} ║ {:8} ║", "Type / Size", "Cutoff");
    for &c in cutoffs {
        print!(" {:6} ║", c);
    }
    println!();
    println!("╠══════════════╬══════════╬════════╬════════╬════════╬════════╬════════════╣");

    for &kind in &kinds {
        for &n in sizes {
            let reps = reps_for(n);
            let base = generate(kind, n);

            let seq_t = median_time(reps, || {
                let mut d = base.clone();
                sequential_quicksort(&mut d);
            });

            print!("║ {:>7}/{:>4} ║ {:>8} ║", kind.label(), format_size(n), fmt_ms(seq_t));

            for &cutoff in cutoffs {
                let par_t = median_time(reps, || {
                    let mut d = base.clone();
                    parallel_quicksort(&mut d, cutoff);
                });
                let speedup = seq_t.as_secs_f64() / par_t.as_secs_f64();
                print!(" {:>5.2}x ║", speedup);
            }
            println!();
        }
        println!("╠══════════════╬══════════╬════════╬════════╬════════╬════════╬════════════╣");
    }
    println!("╚══════════════╩══════════╩════════╩════════╩════════╩════════╩════════════╝");

    println!("\nBenchmark complete.");
}

fn format_size(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        format!("{}", n)
    }
}