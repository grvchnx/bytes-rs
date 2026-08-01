/*!
 * bytes.rs
 * High-performance Rust port of visionmedia/bytes.js
 * Copyright(c) 2012-2014 TJ Holowaychuk
 * Copyright(c) 2015 Jed Watson
 * MIT Licensed
 *
 * Original repository: https://github.com/visionmedia/bytes.js
 */

#![forbid(unsafe_code)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use once_cell::sync::Lazy;
use regex::Regex;

const B: f64 = 1.0;
const KB: f64 = 1024.0;
const MB: f64 = 1048576.0; // 1024^2
const GB: f64 = 1073741824.0; // 1024^3
const TB: f64 = 1099511627776.0; // 1024^4
const PB: f64 = 1125899906842624.0; // 1024^5

static PARSE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^((-|\+)?(\d+(?:\.\d+)?))\s*(kb|mb|gb|tb|pb)$").unwrap()
});

#[derive(Default)]
#[napi(object)]
pub struct FormatOptions {
    pub decimal_places: Option<u32>,
    pub fixed_decimals: Option<bool>,
    pub thousands_separator: Option<String>,
    pub unit: Option<String>,
    pub unit_separator: Option<String>,
}

fn js_parse_int(s: &str) -> Option<f64> {
    let trimmed = s.trim_start();
    let mut chars = trimmed.chars().peekable();

    let mut is_neg = false;
    if let Some(&c) = chars.peek() {
        if c == '-' {
            is_neg = true;
            chars.next();
        } else if c == '+' {
            chars.next();
        }
    }

    let mut num_str = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            num_str.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if num_str.is_empty() {
        return None;
    }

    let val: f64 = num_str.parse().ok()?;
    if is_neg {
        Some(-val)
    } else {
        Some(val)
    }
}

fn format_thousands(int_part: &str, sep: &str) -> String {
    let (is_neg, digits) = if let Some(stripped) = int_part.strip_prefix('-') {
        (true, stripped)
    } else {
        (false, int_part)
    };

    if digits.len() <= 3 {
        return int_part.to_string();
    }

    let len = digits.len();
    let first_len = len % 3;
    let mut chunks = Vec::new();

    if first_len > 0 {
        chunks.push(&digits[..first_len]);
    }

    let mut idx = first_len;
    while idx < len {
        chunks.push(&digits[idx..idx + 3]);
        idx += 3;
    }

    let joined = chunks.join(sep);
    if is_neg {
        format!("-{}", joined)
    } else {
        joined
    }
}

/// Parse a string value into an integer in bytes.
pub fn parse_bytes_str(val: &str) -> Option<f64> {
    if let Some(caps) = PARSE_REGEX.captures(val) {
        let float_str = &caps[1];
        let unit_str = caps[4].to_lowercase();
        let float_val: f64 = float_str.parse().ok()?;
        let unit_mult = match unit_str.as_str() {
            "kb" => KB,
            "mb" => MB,
            "gb" => GB,
            "tb" => TB,
            "pb" => PB,
            _ => B,
        };
        Some((float_val * unit_mult).floor())
    } else {
        let float_val = js_parse_int(val)?;
        Some((float_val * B).floor())
    }
}

/// Format the given value in bytes into a string.
pub fn format_bytes(value: f64, options: Option<&FormatOptions>) -> Option<String> {
    if !value.is_finite() {
        return None;
    }

    let mag = value.abs();

    let thousands_separator = options
        .and_then(|o| o.thousands_separator.as_deref())
        .unwrap_or("");
    let unit_separator = options
        .and_then(|o| o.unit_separator.as_deref())
        .unwrap_or("");
    let decimal_places = options
        .and_then(|o| o.decimal_places)
        .unwrap_or(2);
    let fixed_decimals = options
        .and_then(|o| o.fixed_decimals)
        .unwrap_or(false);
    let input_unit = options
        .and_then(|o| o.unit.as_deref())
        .unwrap_or("");

    let unit_lower = input_unit.to_lowercase();

    let (unit_str, mult) = match unit_lower.as_str() {
        "b" => (if input_unit.is_empty() { "B".to_string() } else { input_unit.to_string() }, B),
        "kb" => (if input_unit.is_empty() { "KB".to_string() } else { input_unit.to_string() }, KB),
        "mb" => (if input_unit.is_empty() { "MB".to_string() } else { input_unit.to_string() }, MB),
        "gb" => (if input_unit.is_empty() { "GB".to_string() } else { input_unit.to_string() }, GB),
        "tb" => (if input_unit.is_empty() { "TB".to_string() } else { input_unit.to_string() }, TB),
        "pb" => (if input_unit.is_empty() { "PB".to_string() } else { input_unit.to_string() }, PB),
        _ => {
            if mag >= PB {
                ("PB".to_string(), PB)
            } else if mag >= TB {
                ("TB".to_string(), TB)
            } else if mag >= GB {
                ("GB".to_string(), GB)
            } else if mag >= MB {
                ("MB".to_string(), MB)
            } else if mag >= KB {
                ("KB".to_string(), KB)
            } else {
                ("B".to_string(), B)
            }
        }
    };

    let val = value / mult;
    let formatted_val = format!("{:.1$}", val, decimal_places as usize);

    let mut str_val = formatted_val;

    if !fixed_decimals {
        if str_val.contains('.') {
            let mut trimmed = str_val.trim_end_matches('0');
            if trimmed.ends_with('.') {
                trimmed = &trimmed[..trimmed.len() - 1];
            }
            str_val = trimmed.to_string();
        }
    }

    if !thousands_separator.is_empty() {
        let parts: Vec<&str> = str_val.split('.').collect();
        let int_part = parts[0];
        let dec_part = if parts.len() > 1 { parts[1] } else { "" };

        let formatted_int = format_thousands(int_part, thousands_separator);
        if !dec_part.is_empty() {
            str_val = format!("{}.{}", formatted_int, dec_part);
        } else {
            str_val = formatted_int;
        }
    }

    Some(format!("{}{}{}", str_val, unit_separator, unit_str))
}

// -----------------------------------------------------------------------------
// N-API Exports for Node.js
// -----------------------------------------------------------------------------

fn parse_format_options(obj: &Object) -> FormatOptions {
    let decimal_places = if let Ok(true) = obj.has_named_property("decimalPlaces") {
        if let Ok(val) = obj.get_named_property::<Unknown>("decimalPlaces") {
            if let Ok(ValueType::Number) = val.get_type() {
                val.coerce_to_number().ok().and_then(|n| n.get_uint32().ok())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let fixed_decimals = if let Ok(true) = obj.has_named_property("fixedDecimals") {
        if let Ok(val) = obj.get_named_property::<Unknown>("fixedDecimals") {
            if let Ok(ValueType::Boolean) = val.get_type() {
                val.coerce_to_bool().ok().map(|b| b.get_value().unwrap_or(false))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let thousands_separator = if let Ok(true) = obj.has_named_property("thousandsSeparator") {
        if let Ok(val) = obj.get_named_property::<Unknown>("thousandsSeparator") {
            if let Ok(ValueType::String) = val.get_type() {
                val.coerce_to_string()
                    .ok()
                    .and_then(|s| s.into_utf8().ok())
                    .and_then(|u| u.into_owned().ok())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let unit = if let Ok(true) = obj.has_named_property("unit") {
        if let Ok(val) = obj.get_named_property::<Unknown>("unit") {
            if let Ok(ValueType::String) = val.get_type() {
                val.coerce_to_string()
                    .ok()
                    .and_then(|s| s.into_utf8().ok())
                    .and_then(|u| u.into_owned().ok())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let unit_separator = if let Ok(true) = obj.has_named_property("unitSeparator") {
        if let Ok(val) = obj.get_named_property::<Unknown>("unitSeparator") {
            if let Ok(ValueType::String) = val.get_type() {
                val.coerce_to_string()
                    .ok()
                    .and_then(|s| s.into_utf8().ok())
                    .and_then(|u| u.into_owned().ok())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    FormatOptions {
        decimal_places,
        fixed_decimals,
        thousands_separator,
        unit,
        unit_separator,
    }
}

#[napi]
pub fn bytes(env: Env, val: Option<Unknown>, options: Option<Object>) -> Result<Option<Unknown>> {
    let u = match val {
        Some(u) => u,
        None => return Ok(None),
    };
    let value_type = u.get_type()?;
    match value_type {
        ValueType::String => {
            let str_val = u.coerce_to_string()?.into_utf8()?.into_owned()?;
            match parse_bytes_str(&str_val) {
                Some(n) => Ok(Some(env.create_double(n)?.into_unknown())),
                None => Ok(None),
            }
        }
        ValueType::Number => {
            let num = u.coerce_to_number()?.get_double()?;
            let opts = options.map(|obj| parse_format_options(&obj));
            match format_bytes(num, opts.as_ref()) {
                Some(s) => Ok(Some(env.create_string(&s)?.into_unknown())),
                None => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

#[napi]
pub fn parse(val: Option<Unknown>) -> Result<Option<f64>> {
    let u = match val {
        Some(u) => u,
        None => return Ok(None),
    };
    let value_type = u.get_type()?;
    match value_type {
        ValueType::Number => {
            let num = u.coerce_to_number()?.get_double()?;
            if num.is_nan() {
                Ok(None)
            } else {
                Ok(Some(num))
            }
        }
        ValueType::String => {
            let str_val = u.coerce_to_string()?.into_utf8()?.into_owned()?;
            Ok(parse_bytes_str(&str_val))
        }
        _ => Ok(None),
    }
}

#[napi]
pub fn format(val: Option<Unknown>, options: Option<Object>) -> Result<Option<String>> {
    let u = match val {
        Some(u) => u,
        None => return Ok(None),
    };
    let value_type = u.get_type()?;
    if value_type != ValueType::Number {
        return Ok(None);
    }
    let num = u.coerce_to_number()?.get_double()?;
    let opts = options.map(|obj| parse_format_options(&obj));
    Ok(format_bytes(num, opts.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        assert_eq!(format_bytes(0.0, None).unwrap().to_lowercase(), "0b");
        assert_eq!(format_bytes(100.0, None).unwrap().to_lowercase(), "100b");
        assert_eq!(format_bytes(-100.0, None).unwrap().to_lowercase(), "-100b");
        assert_eq!(format_bytes(1024.0, None).unwrap().to_lowercase(), "1kb");
        assert_eq!(format_bytes(-1024.0, None).unwrap().to_lowercase(), "-1kb");
        assert_eq!(format_bytes(1048576.0, None).unwrap().to_lowercase(), "1mb");
        assert_eq!(format_bytes(1073741824.0, None).unwrap().to_lowercase(), "1gb");
        assert_eq!(format_bytes(1099511627776.0, None).unwrap().to_lowercase(), "1tb");
        assert_eq!(format_bytes(1125899906842624.0, None).unwrap().to_lowercase(), "1pb");

        // Standard case
        assert_eq!(format_bytes(10.0, None).unwrap(), "10B");
        assert_eq!(format_bytes(1024.0, None).unwrap(), "1KB");
        assert_eq!(format_bytes(1048576.0, None).unwrap(), "1MB");

        // Options
        let opts = FormatOptions {
            thousands_separator: Some(" ".to_string()),
            ..Default::default()
        };
        assert_eq!(format_bytes(1000.0, Some(&opts)).unwrap(), "1 000B");

        let opts = FormatOptions {
            unit_separator: Some(" ".to_string()),
            ..Default::default()
        };
        assert_eq!(format_bytes(1024.0, Some(&opts)).unwrap(), "1 KB");

        let opts = FormatOptions {
            decimal_places: Some(3),
            fixed_decimals: Some(true),
            ..Default::default()
        };
        assert_eq!(format_bytes(1024.0, Some(&opts)).unwrap().to_lowercase(), "1.000kb");
    }

    #[test]
    fn test_parse() {
        assert_eq!(parse_bytes_str("1kb"), Some(1024.0));
        assert_eq!(parse_bytes_str("1KB"), Some(1024.0));
        assert_eq!(parse_bytes_str("0.5kb"), Some(512.0));
        assert_eq!(parse_bytes_str("1.5kb"), Some(1536.0));
        assert_eq!(parse_bytes_str("1mb"), Some(1048576.0));
        assert_eq!(parse_bytes_str("1gb"), Some(1073741824.0));
        assert_eq!(parse_bytes_str("1tb"), Some(1099511627776.0));
        assert_eq!(parse_bytes_str("1pb"), Some(1125899906842624.0));
        assert_eq!(parse_bytes_str("0"), Some(0.0));
        assert_eq!(parse_bytes_str("-1"), Some(-1.0));
        assert_eq!(parse_bytes_str("1024"), Some(1024.0));
        assert_eq!(parse_bytes_str("0x11"), Some(0.0));
        assert_eq!(parse_bytes_str("foobar"), None);
        assert_eq!(parse_bytes_str("1.1b"), Some(1.0));
        assert_eq!(parse_bytes_str("1.0001kb"), Some(1024.0));
        assert_eq!(parse_bytes_str("1 TB"), Some(1099511627776.0));
    }

    #[test]
    fn bench_performance() {
        use std::time::Instant;

        let iterations = 500_000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = parse_bytes_str("1.5GB");
            let _ = format_bytes(1610612736.0, None);
        }
        let elapsed = start.elapsed();
        let total_ops = iterations * 2;
        let ns_per_op = elapsed.as_nanos() / total_ops as u128;
        println!(
            "\n⚡ [RUST BENCHMARK] Executed {} operations in {:?} (~{} ns/op, {:.2} million ops/sec)",
            total_ops,
            elapsed,
            ns_per_op,
            (total_ops as f64 / elapsed.as_secs_f64()) / 1_000_000.0
        );
    }
}
