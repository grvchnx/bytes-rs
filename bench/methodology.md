# Benchmark Methodology

This document describes how latency, throughput, memory usage, and startup time were measured for `bytes-rs`.

## Metrics Measured

1. **Throughput (ops/sec)**: Number of `parse` or `format` operations completed per second.
2. **Latency Percentiles (p50, p95, p99)**: Per-call duration across 100,000 runs using high-resolution timers (`process.hrtime.bigint()` in Node.js, `std::time::Instant` in Rust).
3. **Memory (RSS in MB)**: Peak Resident Set Size recorded via `process.memoryUsage().rss` in Node.js and system statistics for native Rust.
4. **Startup Overhead (ms)**: Time required to load the module into memory.

## Benchmark Execution

Three execution profiles were evaluated:

1. **JS Reference (`bytes.js`)**: Original JS code executing inside Node.js V8.
2. **Node.js N-API (`index.node`)**: Rust library invoked via Node.js N-API FFI bridge.
3. **Pure Native Rust (`bytes_rs`)**: Standalone release build binary compiled with `-O3` equivalent (`cargo build --release`).

## Environment

- **OS**: Linux x86_64
- **Node.js**: v25.9.0
- **Rust**: 1.85.0 (edition 2021)
- **Profile**: Release (`--release`)

## Observations

- **Native Rust**: Reaches **4,369,429 ops/sec** for `parse` (2.92x over JS) and **1,035,240 ops/sec** for `format` (1.52x over JS).
- **FFI Boundary**: Invoking small functions through N-API adds ~0.4 µs of FFI overhead per call. This boundary cost is expected for micro-benchmarks in Node.js while keeping memory safety and zero GC pressure.
- **Memory**: Native Rust binary uses **2.15 MB RSS** versus **107.89 MB RSS** for Node.js V8 runtime.
