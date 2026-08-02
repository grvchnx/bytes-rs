#![forbid(unsafe_code)]

use bytes_rs::{format_bytes, parse_number, parse_string, FormatOptions};
use proptest::prelude::*;

// Differential & Property Fuzzing Harness for Rust bytes-rs crate
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn fuzz_parse_string(s in "\\PC*") {
        let _ = parse_string(&s);
    }

    #[test]
    fn fuzz_parse_number(val in proptest::num::f64::ANY) {
        let _ = parse_number(val);
    }

    #[test]
    fn fuzz_format(
        val in proptest::num::f64::ANY,
        decimal_places in proptest::option::of(0u32..20u32),
        fixed_decimals in proptest::option::of(any::<bool>()),
        thousands_separator in proptest::option::of("[., _-]?"),
        unit in proptest::option::of("(b|kb|mb|gb|tb|pb|B|KB|MB|GB|TB|PB|invalid)?"),
        unit_separator in proptest::option::of("[ \t_-]?")
    ) {
        let opts = FormatOptions {
            decimal_places,
            fixed_decimals,
            thousands_separator,
            unit,
            unit_separator,
        };
        let _ = format_bytes(val, Some(opts));
    }
}
