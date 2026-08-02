#![forbid(unsafe_code)]

use regex::Regex;
use std::sync::OnceLock;

#[cfg(feature = "napi-bindings")]
use napi::bindgen_prelude::*;
#[cfg(feature = "napi-bindings")]
use napi_derive::napi;

static PARSE_REGEXP: OnceLock<Regex> = OnceLock::new();
static FORMAT_DECIMALS_REGEXP: OnceLock<Regex> = OnceLock::new();

const MAP_B: f64 = 1.0;
const MAP_KB: f64 = 1024.0;
const MAP_MB: f64 = 1048576.0;
const MAP_GB: f64 = 1073741824.0;
const MAP_TB: f64 = 1099511627776.0;
const MAP_PB: f64 = 1125899906842624.0;

fn get_unit_map_entry(unit: &str) -> Option<(&'static str, f64)> {
    match unit.to_lowercase().as_str() {
        "b" => Some(("b", MAP_B)),
        "kb" => Some(("kb", MAP_KB)),
        "mb" => Some(("mb", MAP_MB)),
        "gb" => Some(("gb", MAP_GB)),
        "tb" => Some(("tb", MAP_TB)),
        "pb" => Some(("pb", MAP_PB)),
        _ => None,
    }
}

/// JS parseInt(val, 10) behavior when parseRegExp does not match.
fn js_parse_int_10(s: &str) -> Option<f64> {
    let trimmed = s.trim_start();
    if trimmed.is_empty() {
        return None;
    }
    let mut chars = trimmed.char_indices().peekable();
    let mut sign = 1.0;
    if let Some((_, c)) = chars.peek() {
        if *c == '-' {
            sign = -1.0;
            chars.next();
        } else if *c == '+' {
            chars.next();
        }
    }
    let mut digits_found = false;
    let mut val: f64 = 0.0;
    for (_, c) in chars {
        if let Some(digit) = c.to_digit(10) {
            digits_found = true;
            val = val * 10.0 + digit as f64;
        } else {
            break;
        }
    }
    if digits_found {
        Some(sign * val)
    } else {
        None
    }
}

/// Parse raw number into bytes count.
pub fn parse_number(val: f64) -> Option<f64> {
    if val.is_nan() {
        None
    } else {
        Some(val)
    }
}

/// Parse string into bytes count.
pub fn parse_string(val: &str) -> Option<f64> {
    let re = PARSE_REGEXP.get_or_init(|| {
        Regex::new(r"(?i)^((-|\+)?(\d+(?:\.\d+)?)) *(kb|mb|gb|tb|pb)$").unwrap()
    });

    if let Some(caps) = re.captures(val) {
        let float_val: f64 = caps.get(1)?.as_str().parse().ok()?;
        let unit_str = caps.get(4)?.as_str();
        let (_, multiplier) = get_unit_map_entry(unit_str)?;
        Some((multiplier * float_val).floor())
    } else {
        let float_val = js_parse_int_10(val)?;
        Some((MAP_B * float_val).floor())
    }
}

#[derive(Debug, Clone, Default)]
pub struct FormatOptions {
    pub decimal_places: Option<u32>,
    pub fixed_decimals: Option<bool>,
    pub thousands_separator: Option<String>,
    pub unit: Option<String>,
    pub unit_separator: Option<String>,
}

fn apply_thousands_separator(s: &str, sep: &str) -> String {
    if sep.is_empty() {
        return s.to_string();
    }
    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];

    let (prefix, digits) = if let Some(stripped) = int_part.strip_prefix('-') {
        ("-", stripped)
    } else if let Some(stripped) = int_part.strip_prefix('+') {
        ("+", stripped)
    } else {
        ("", int_part)
    };

    if digits.len() <= 3 {
        return s.to_string();
    }

    let mut formatted_int = String::with_capacity(int_part.len() + (digits.len() / 3) * sep.len());
    formatted_int.push_str(prefix);

    let first_group_len = digits.len() % 3;
    let (first, rest) = if first_group_len == 0 {
        ("", digits)
    } else {
        digits.split_at(first_group_len)
    };

    if !first.is_empty() {
        formatted_int.push_str(first);
    }

    for (i, chunk) in rest.as_bytes().chunks(3).enumerate() {
        if !first.is_empty() || i > 0 {
            formatted_int.push_str(sep);
        }
        formatted_int.push_str(std::str::from_utf8(chunk).unwrap());
    }

    if parts.len() > 1 {
        format!("{}.{}", formatted_int, parts[1..].join("."))
    } else {
        formatted_int
    }
}

pub fn format_bytes(value: f64, options: Option<FormatOptions>) -> Option<String> {
    if !value.is_finite() {
        return None;
    }

    let opts = options.unwrap_or_default();
    let mag = value.abs();

    let thousands_separator = opts.thousands_separator.unwrap_or_default();
    let unit_separator = opts.unit_separator.unwrap_or_default();
    let decimal_places = opts.decimal_places.unwrap_or(2);
    let fixed_decimals = opts.fixed_decimals.unwrap_or(false);
    let raw_unit = opts.unit.unwrap_or_default();

    let (unit, bytes_per_unit) = match get_unit_map_entry(&raw_unit) {
        Some((_, u_bytes)) => (raw_unit, u_bytes),
        None => {
            if mag >= MAP_PB {
                ("PB".to_string(), MAP_PB)
            } else if mag >= MAP_TB {
                ("TB".to_string(), MAP_TB)
            } else if mag >= MAP_GB {
                ("GB".to_string(), MAP_GB)
            } else if mag >= MAP_MB {
                ("MB".to_string(), MAP_MB)
            } else if mag >= MAP_KB {
                ("KB".to_string(), MAP_KB)
            } else {
                ("B".to_string(), MAP_B)
            }
        }
    };

    let val = value / bytes_per_unit;
    let mut str_val = format!("{:.1$}", val, decimal_places as usize);

    if !fixed_decimals {
        let re = FORMAT_DECIMALS_REGEXP
            .get_or_init(|| Regex::new(r"(?:\.0*|(\.[^0]+)0+)$").unwrap());
        str_val = re.replace(&str_val, "$1").to_string();
    }

    if !thousands_separator.is_empty() {
        str_val = apply_thousands_separator(&str_val, &thousands_separator);
    }

    Some(format!("{}{}{}", str_val, unit_separator, unit))
}

// N-API Bridge Exports

#[cfg(feature = "napi-bindings")]
#[napi(object)]
#[derive(Default)]
pub struct NapiFormatOptions {
    pub decimal_places: Option<u32>,
    pub fixed_decimals: Option<bool>,
    pub thousands_separator: Option<String>,
    pub unit: Option<String>,
    pub unit_separator: Option<String>,
}

#[cfg(feature = "napi-bindings")]
#[napi(js_name = "parse")]
pub fn napi_parse(val: Unknown) -> Result<Option<f64>> {
    match val.get_type()? {
        ValueType::Number => {
            let num = val.coerce_to_number()?.get_double()?;
            Ok(parse_number(num))
        }
        ValueType::String => {
            let s = val.coerce_to_string()?.into_utf8()?.into_owned()?;
            Ok(parse_string(&s))
        }
        _ => Ok(None),
    }
}

#[cfg(feature = "napi-bindings")]
#[napi(js_name = "format")]
pub fn napi_format(
    val: Unknown,
    options: Option<NapiFormatOptions>,
) -> Result<Option<String>> {
    if val.get_type()? != ValueType::Number {
        return Ok(None);
    }
    let num = val.coerce_to_number()?.get_double()?;
    let opts = options.map(|o| FormatOptions {
        decimal_places: o.decimal_places,
        fixed_decimals: o.fixed_decimals,
        thousands_separator: o.thousands_separator,
        unit: o.unit,
        unit_separator: o.unit_separator,
    });
    Ok(format_bytes(num, opts))
}

#[cfg(feature = "napi-bindings")]
#[napi(js_name = "bytes")]
pub fn napi_bytes(
    val: Unknown,
    options: Option<NapiFormatOptions>,
) -> Result<Option<Either<f64, String>>> {
    match val.get_type()? {
        ValueType::String => {
            let s = val.coerce_to_string()?.into_utf8()?.into_owned()?;
            match parse_string(&s) {
                Some(n) => Ok(Some(Either::A(n))),
                None => Ok(None),
            }
        }
        ValueType::Number => {
            let num = val.coerce_to_number()?.get_double()?;
            let opts = options.map(|o| FormatOptions {
                decimal_places: o.decimal_places,
                fixed_decimals: o.fixed_decimals,
                thousands_separator: o.thousands_separator,
                unit: o.unit,
                unit_separator: o.unit_separator,
            });
            match format_bytes(num, opts) {
                Some(s) => Ok(Some(Either::B(s))),
                None => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_raw_numbers() {
        assert_eq!(parse_number(0.0), Some(0.0));
        assert_eq!(parse_number(-1.0), Some(-1.0));
        assert_eq!(parse_number(1.0), Some(1.0));
        assert_eq!(parse_number(10.5), Some(10.5));
        assert_eq!(parse_number(f64::NAN), None);
    }

    #[test]
    fn test_parse_units() {
        assert_eq!(parse_string("1kb"), Some(1024.0));
        assert_eq!(parse_string("1KB"), Some(1024.0));
        assert_eq!(parse_string("0.5kb"), Some(512.0));
        assert_eq!(parse_string("1.5TB"), Some(1.5 * 1024.0 * 1024.0 * 1024.0 * 1024.0));
        assert_eq!(parse_string("1.1b"), Some(1.0));
        assert_eq!(parse_string("1.0001kb"), Some(1024.0));
        assert_eq!(parse_string("0x11"), Some(0.0));
        assert_eq!(parse_string("foobar"), None);
    }

    #[test]
    fn test_format() {
        assert_eq!(format_bytes(0.0, None), Some("0B".to_string()));
        assert_eq!(format_bytes(1024.0, None), Some("1KB".to_string()));
        assert_eq!(
            format_bytes(
                1000.0,
                Some(FormatOptions {
                    thousands_separator: Some(" ".to_string()),
                    ..Default::default()
                })
            ),
            Some("1 000B".to_string())
        );
    }
}
