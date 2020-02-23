use lazy_static::lazy_static;
use regex::{CaptureMatches, Captures, Regex};

#[cfg(feature = "serde")]
use serde::ser::{Serialize, Serializer};

use std::collections::BTreeMap as Map;

#[derive(Debug)]
/// A tagged union representing all
/// possible values in Lua.
pub enum LuaValue {
    Map(Map<String, LuaValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

impl LuaValue {
    fn try_to_string(&self) -> Result<String, &'static str> {
        Ok(match self {
            LuaValue::String(v) => v.clone(),
            LuaValue::Number(v) => v.to_string(),
            LuaValue::Boolean(v) => v.to_string(),
            LuaValue::Null => "nil".into(),
            LuaValue::Map(_) => return Err("can't convert a map into a string"),
        })
    }
}

#[cfg(feature = "serde")]
impl Serialize for LuaValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        match self {
            LuaValue::String(s) => serializer.serialize_str(s),
            LuaValue::Number(n) => serializer.serialize_f64(*n),
            LuaValue::Boolean(b) => serializer.serialize_bool(*b),
            LuaValue::Null => serializer.serialize_none(),
            LuaValue::Map(m) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

fn deserialize_string_replacer<'t>(captures: &Captures<'t>) -> String {
    if &captures[1] < "~\x7A" {
        format!("{}", (&captures[1].as_bytes()[1] - 64) as char)
    } else {
        match &captures[1] {
            "~\x7A" => "\x1E".into(),
            "~\x7B" => "\x7F".into(),
            "~\x7C" => "\x7E".into(),
            "~\x7D" => "\x5E".into(),
            _ => panic!("invalid escape character"),
        }
    }
}

fn deserialize_string<'t>(data: &'t str) -> Result<String, &'static str> {
    lazy_static! {
        static ref REPLACE_REGEX: Regex = Regex::new("(~.)").unwrap();
    }

    // FIXME: figure out if it's possible to have a replacer
    // that returns Result<T, E> instead of relying on panic!()
    std::panic::catch_unwind(|| {
        REPLACE_REGEX
            .replace_all(data, deserialize_string_replacer)
            .into()
    })
    .map_err(|_| "invalid escape character")
}

fn deserialize_number<'t>(data: &'t str) -> Result<f64, &'static str> {
    match data {
        "-1.#INF" | "-inf" => Ok(std::f64::NEG_INFINITY),
        "1.#INF" | "inf" => Ok(std::f64::INFINITY),
        "1.#IND" | "nan" => Ok(std::f64::NAN), // 0x7ff8000000000000
        "-1.#IND" | "-nan" => Ok(-std::f64::NAN), // 0xfff8000000000000
        v => v.parse().map_err(|_| "failed parsing a number"),
    }
}

fn deserialize_value<'r, 't>(
    iter: &mut CaptureMatches<'r, 't>,
    capture: Captures<'t>,
) -> Result<Option<LuaValue>, &'static str> {
    Ok(match &capture["control"] {
        "^^" => None,
        "^S" => Some(LuaValue::String(deserialize_string(&capture["data"])?)),
        "^N" => Some(LuaValue::Number(deserialize_number(&capture["data"])?)),
        "^F" => Some(iter.next().ok_or_else(|| "missing exponent").and_then(
            |ref next_capture| {
                if &next_capture["control"] == "^f" {
                    let mantissa = deserialize_number(&capture["data"])?;
                    let exponent = deserialize_number(&next_capture["data"])?;

                    Ok(LuaValue::Number(mantissa * (2f64.powf(exponent))))
                } else {
                    Err("missing exponent")
                }
            },
        )?),
        "^B" => Some(LuaValue::Boolean(true)),
        "^b" => Some(LuaValue::Boolean(false)),
        "^Z" => Some(LuaValue::Null),
        "^T" => {
            let mut map = Map::new();

            loop {
                match iter.next() {
                    Some(next_capture) => {
                        if &next_capture["control"] == "^t" {
                            break;
                        }

                        let key =
                            deserialize_value(iter, next_capture)?.ok_or_else(|| "invalid key")?;

                        let value_capture = iter.next().ok_or_else(|| "missing value")?;
                        let value = deserialize_value(iter, value_capture)?
                            .ok_or_else(|| "invalid value")?;

                        map.insert(key.try_to_string().map_err(|_| "invalid key")?, value);
                    }
                    None => return Err("unexpected end of a table"),
                }
            }

            Some(LuaValue::Map(map))
        }
        _ => return Err("unknown control sequence"),
    })
}

pub(crate) fn deserialize(data: &str) -> Result<Vec<LuaValue>, &'static str> {
    lazy_static! {
        static ref CAPTURE_REGEX: Regex = Regex::new(r"(?P<control>\^.)(?P<data>[^^]*)").unwrap();
    }

    let mut iter = CAPTURE_REGEX.captures_iter(data);

    match iter.next() {
        Some(ref capture) if &capture["control"] == "^1" => (),
        _ => return Err("supplied data is not AceSerializer data (rev 1)"),
    };

    let mut data = Vec::new();
    while let Some(capture) = iter.next() {
        if let Some(value) = deserialize_value(&mut iter, capture)? {
            data.push(value);
        }
    }

    Ok(data)
}
