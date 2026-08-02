# Architecture & Porting Decisions (DECISIONS.md)

This document details the architectural choices, trade-offs, and behavioral nuances encountered while porting `bytes.js` to Rust (`bytes-rs`).

---

## 1. Dual Target Architecture (`rlib` + `cdylib`)

### Choice
The crate is configured with `crate-type = ["cdylib", "rlib"]` behind an optional `napi-bindings` feature flag.

### Context
- **Pure Rust Crate (`rlib`)**: Rust applications import `bytes_rs` directly without any Node.js overhead or FFI dependencies when using `--no-default-features`.
- **Node.js Addon (`cdylib`)**: Node.js imports `index.node` via `napi-rs` to preserve full API parity with `bytes.js`.

---

## 2. Memory Safety Guarantee (`#![forbid(unsafe_code)]`)

### Choice
`#![forbid(unsafe_code)]` is declared at the crate root of `src/lib.rs` and in test modules.

### Context
All parsing, string manipulation, regular expression matching, and formatting logic is written in safe Rust. The N-API layer delegates memory allocation and object creation safely via `napi-rs` abstractions.

---

## 3. Regular Expression Caching with `OnceLock`

### Choice
Static regex patterns (`PARSE_REGEXP` and `FORMAT_DECIMALS_REGEXP`) are initialized lazily with `std::sync::OnceLock`.

### Context
Compiling regular expressions on every `parse` or `format` call incurs unnecessary heap allocations. Using `OnceLock` ensures thread-safe, single-initialization overhead.

---

## 4. Custom `parseInt(val, 10)` Parser

### Choice
Implemented `js_parse_int_10` to replicate JavaScript's permissive `parseInt(val, 10)` behavior when unit suffixes are not present.

### Context
JavaScript's `parseInt("1024foo", 10)` evaluates to `1024` by reading leading digits until encountering a non-numeric character. `js_parse_int_10` mirrors this behavior by scanning optional signs, skipping leading whitespace, and consuming decimal digits into `f64`.

---

## 5. Option Handling and `null` / `undefined` Normalization

### Choice
An option sanitizer in `index.js` converts `null` property values on options objects into `undefined` before handing them to N-API functions.

### Context
In JS `bytes.js`, passing `{ thousandsSeparator: null }` falls back to empty string `""` due to JS falsy check `(options && options.thousandsSeparator) || ''`. N-API object deserialization treats `null` differently from `undefined`. Sanitizing `null` to `undefined` maintains 100% test compatibility with the original JS Mocha suite.

---

## 6. Floating-Point Rounding Analysis (IEEE 754)

### Observation
During differential fuzzing over 5.9M iterations, a least-significant digit difference was observed on large floats formatted with high decimal precision (`decimalPlaces >= 10` on inputs near 10^11):

- **JS V8 (`toFixed(10)`)**: `"-300 015 247 373.0024414063_B"`
- **Rust (`format!("{:.1$}")`)**: `"-300 015 247 373.0024414062_B"`

### Decision
This discrepancy stems from differences between V8's `Number.prototype.toFixed` string conversion algorithm and Rust stdlib `format!` half-to-even float rounding. Standard Rust stdlib formatting was retained to maintain idiomatic Rust behavior.
