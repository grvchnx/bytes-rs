use bytes_rs::{format_bytes, parse_string};
use std::time::Instant;

fn main() {
    let parse_inputs = vec!["1024", "1.5MB", "2.5GB", "100TB", "500B", "0.5KB", "-1024MB", "10PB"];
    let format_inputs = vec![1024.0, 1572864.0, 2684354560.0, 1099511627776.0, 500.0, 512.0, -1073741824.0];

    let iterations = 10_000_000;

    // Warmup
    for i in 0..100_000 {
        let _ = parse_string(parse_inputs[i % parse_inputs.len()]);
        let _ = format_bytes(format_inputs[i % format_inputs.len()], None);
    }

    // Benchmark Native Rust Parse
    let start_parse = Instant::now();
    for i in 0..iterations {
        let _ = parse_string(parse_inputs[i % parse_inputs.len()]);
    }
    let duration_parse = start_parse.elapsed();
    let parse_ops_sec = (iterations as f64 / duration_parse.as_secs_f64()) as u64;

    // Benchmark Native Rust Format
    let start_format = Instant::now();
    for i in 0..iterations {
        let _ = format_bytes(format_inputs[i % format_inputs.len()], None);
    }
    let duration_format = start_format.elapsed();
    let format_ops_sec = (iterations as f64 / duration_format.as_secs_f64()) as u64;

    println!("=== Pure Native Rust Performance ===");
    println!("Native Rust Parse Throughput:  {:?} ops/sec ({:?} for {} ops)", parse_ops_sec, duration_parse, iterations);
    println!("Native Rust Format Throughput: {:?} ops/sec ({:?} for {} ops)", format_ops_sec, duration_format, iterations);
    println!("Average Parse Latency:         {:.3} ns", (duration_parse.as_nanos() as f64) / (iterations as f64));
    println!("Average Format Latency:        {:.3} ns", (duration_format.as_nanos() as f64) / (iterations as f64));
}
