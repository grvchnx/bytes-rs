#![forbid(unsafe_code)]

use bytes_rs::{format_bytes, parse_number, parse_string, FormatOptions};
use proptest::prelude::*;

proptest! {
    #[test]
    fn fuzz_parse_string(s in "\\PC*") {
        // Ensure parsing arbitrary random strings never panics
        let _ = parse_string(&s);
    }

    #[test]
    fn fuzz_parse_number(val in proptest::num::f64::ANY) {
        // Ensure parsing numbers never panics
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
        // Ensure formatting arbitrary inputs with arbitrary options never panics
        let _ = format_bytes(val, Some(opts));
    }

    #[test]
    fn roundtrip_format_parse_identity(
        val in 0.0f64..1_000_000_000_000f64
    ) {
        if let Some(formatted) = format_bytes(val, None) {
            if let Some(parsed) = parse_string(&formatted) {
                // Check relative error or exact match for roundtripped formatted values
                let diff = (val - parsed).abs();
                prop_assert!(diff <= 1024.0 || diff / (val + 1.0) < 0.05);
            }
        }
    }
}
