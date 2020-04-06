mod reader;

use crate::value::{LuaMapKey, LuaValue, Map};
use reader::StrReader;

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
                                let key = self.deserialize_helper()?.ok_or("missing key").and_then(LuaMapKey::from_value)?;
                                let value = match self.reader.peek_identifier()? {
                                    "^t" => return Err("unexpected end of a table"),
                                    _ => self.deserialize_helper()?.ok_or("missing value")?,
                                };
                                map.insert(key, value);
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
