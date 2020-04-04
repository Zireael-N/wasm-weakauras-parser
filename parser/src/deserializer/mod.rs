mod reader;
use reader::StrReader;

#[cfg(all(not(feature = "indexmap"), feature = "fnv"))]
use fnv::FnvHashMap as Map;
#[cfg(feature = "indexmap")]
use indexmap::IndexMap as Map;
#[cfg(not(any(feature = "indexmap", feature = "fnv")))]
use std::collections::BTreeMap as Map;

#[cfg(feature = "serde")]
use serde::ser::{Serialize, Serializer};

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

pub struct Deserializer<'s> {
    remaining_depth: usize,
    reader: StrReader<'s>,
}

impl<'s> Deserializer<'s> {
    pub fn from_str(slice: &'s str) -> Self {
        Self {
            remaining_depth: 128,
            reader: StrReader::new(slice),
        }
    }

    /// Returns an array of deserialized values
    #[allow(dead_code)]
    pub fn deserialize(mut self) -> Result<Vec<LuaValue>, &'static str> {
        self.reader.read_identifier().and_then(|v| {
            if v == "^1" {
                Ok(())
            } else {
                Err("supplied data is not AceSerializer data (rev 1)")
            }
        })?;

        let mut result = Vec::new();

        while self.reader.peek_identifier().is_ok() {
            if let Some(v) = self.deserialize_helper()? {
                result.push(v);
            }
        }

        Ok(result)
    }

    /// Returns the first deserialized value
    #[allow(dead_code)]
    pub fn deserialize_first(mut self) -> Result<Option<LuaValue>, &'static str> {
        self.reader.read_identifier().and_then(|v| {
            if v == "^1" {
                Ok(())
            } else {
                Err("supplied data is not AceSerializer data (rev 1)")
            }
        })?;

        self.deserialize_helper()
    }

    fn deserialize_helper(&mut self) -> Result<Option<LuaValue>, &'static str> {
        // Taken from serde_json
        macro_rules! check_recursion {
            ($($body:tt)*) => {
                self.remaining_depth -= 1;
                if self.remaining_depth == 0 {
                    return Err("recursion limit exceeded");
                }

                $($body)*

                self.remaining_depth += 1;
            }
        }

        Ok(Some(match self.reader.read_identifier()? {
            "^^" => return Ok(None),
            "^Z" => LuaValue::Null,
            "^B" => LuaValue::Boolean(true),
            "^b" => LuaValue::Boolean(false),
            "^S" => LuaValue::String(String::from(self.reader.parse_str()?)),
            "^N" => LuaValue::Number(
                self.reader
                    .read_until_next()
                    .and_then(Self::deserialize_number)?,
            ),
            "^F" => {
                let mantissa = self
                    .reader
                    .read_until_next()
                    .and_then(|v| v.parse::<f64>().map_err(|_| "failed to parse a number"))?;
                let exponent = match self.reader.read_identifier()? {
                    "^f" => self
                        .reader
                        .read_until_next()
                        .and_then(|v| v.parse::<f64>().map_err(|_| "failed to parse a number"))?,
                    _ => return Err("missing exponent"),
                };

                LuaValue::Number(mantissa * (2f64.powf(exponent)))
            }
            "^T" => {
                let mut map = Map::default();
                loop {
                    match self.reader.peek_identifier()? {
                        "^t" => {
                            let _ = self.reader.read_identifier();
                            break;
                        }
                        _ => {
                            check_recursion! {
                                let key = self.deserialize_helper()?.ok_or("missing key")?;
                                let value = match self.reader.peek_identifier()? {
                                    "^t" => return Err("unexpected end of a table"),
                                    _ => self.deserialize_helper()?.ok_or("missing value")?,
                                };
                                map.insert(key.try_to_string()?, value);
                            }
                        }
                    }
                }
                LuaValue::Map(map)
            }
            _ => return Err("invalid identifier"),
        }))
    }

    fn deserialize_number(data: &str) -> Result<f64, &'static str> {
        match data {
            "1.#INF" | "inf" => Ok(std::f64::INFINITY),
            "-1.#INF" | "-inf" => Ok(std::f64::NEG_INFINITY),
            v => v.parse().map_err(|_| "failed to parse a number"),
        }
    }
}
