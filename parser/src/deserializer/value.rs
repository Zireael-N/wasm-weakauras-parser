use super::Map;

#[cfg(feature = "serde")]
use serde::ser::{Serialize, Serializer};

#[derive(Debug)]
/// A tagged union representing all
/// possible values in Lua.
pub enum LuaValue {
    Map(Map<LuaMapKey, LuaValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

pub struct LuaMapKey(LuaValue);
impl LuaMapKey {
    pub fn from_value(value: LuaValue) -> Result<Self, &'static str> {
        if let LuaValue::Null = value {
            Err("map key can't be null")
        } else {
            Ok(Self(value))
        }
    }
}

use core::hash::{Hash, Hasher};
impl Hash for LuaValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            LuaValue::Map(m) => {
                for (k, v) in m {
                    k.hash(state);
                    v.hash(state);
                }
            }
            LuaValue::String(s) => s.hash(state),
            LuaValue::Number(n) => state.write_u64(n.to_bits()),
            LuaValue::Boolean(b) => b.hash(state),
            LuaValue::Null => state.write_u8(0),
        }
    }
}

use std::cmp::Ordering;
impl PartialOrd for LuaValue {
    // Number > String > Boolean > Map > Null
    fn partial_cmp(&self, other: &LuaValue) -> Option<Ordering> {
        Some(match (self, other) {
            (LuaValue::Number(n1), LuaValue::Number(n2)) => {
                n1.partial_cmp(n2)
                    .unwrap_or_else(|| match (n1.is_nan(), n2.is_nan()) {
                        (true, false) => Ordering::Less,
                        (false, true) => Ordering::Greater,
                        _ => Ordering::Equal,
                    })
            }
            (LuaValue::Number(_), _) => Ordering::Greater,
            (_, LuaValue::Number(_)) => Ordering::Less,
            (LuaValue::String(s1), LuaValue::String(s2)) => s1.cmp(s2),
            (LuaValue::String(_), LuaValue::Boolean(_))
            | (LuaValue::String(_), LuaValue::Map(_)) => Ordering::Greater,
            (LuaValue::Boolean(_), LuaValue::String(_))
            | (LuaValue::Map(_), LuaValue::String(_)) => Ordering::Less,
            (LuaValue::Boolean(b1), LuaValue::Boolean(b2)) => b1.cmp(b2),
            (LuaValue::Boolean(_), LuaValue::Map(_)) => Ordering::Greater,
            (LuaValue::Map(_), LuaValue::Boolean(_)) => Ordering::Less,
            (LuaValue::Map(m1), LuaValue::Map(m2)) => {
                let mut entries1: Vec<_> = m1.iter().collect();
                let mut entries2: Vec<_> = m2.iter().collect();

                entries1.sort_unstable();
                entries2.sort_unstable();

                entries1.cmp(&entries2)
            }
            (LuaValue::Null, LuaValue::Null) => Ordering::Equal,
            (LuaValue::Null, _) => Ordering::Less,
            (_, LuaValue::Null) => Ordering::Greater,
        })
    }
}
impl Ord for LuaValue {
    fn cmp(&self, other: &LuaValue) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl PartialEq for LuaValue {
    fn eq(&self, other: &LuaValue) -> bool {
        match (self, other) {
            (LuaValue::Map(m1), LuaValue::Map(m2)) => m1.eq(m2),
            (LuaValue::String(s1), LuaValue::String(s2)) => s1.eq(s2),
            (LuaValue::Number(n1), LuaValue::Number(n2)) => n1.eq(n2),
            (LuaValue::Boolean(b1), LuaValue::Boolean(b2)) => b1.eq(b2),
            (LuaValue::Null, LuaValue::Null) => true,
            _ => false,
        }
    }
}
impl Eq for LuaValue {}

impl Hash for LuaMapKey {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}
impl PartialOrd for LuaMapKey {
    #[inline(always)]
    fn partial_cmp(&self, other: &LuaMapKey) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl Ord for LuaMapKey {
    #[inline(always)]
    fn cmp(&self, other: &LuaMapKey) -> Ordering {
        self.0.cmp(&other.0)
    }
}
impl PartialEq for LuaMapKey {
    #[inline(always)]
    fn eq(&self, other: &LuaMapKey) -> bool {
        self.0.eq(&other.0)
    }
}
impl Eq for LuaMapKey {}

use std::borrow::Cow;
impl LuaMapKey {
    fn to_string(&self) -> Cow<'_, str> {
        match self.0 {
            LuaValue::String(ref v) => Cow::from(v),
            LuaValue::Number(v) => Cow::from(v.to_string()),
            LuaValue::Boolean(v) => Cow::from(v.to_string()),
            LuaValue::Map(ref m) => Cow::from(format!("map at {:p}", m)),
            LuaValue::Null => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

use std::fmt::{self, Debug};
impl Debug for LuaMapKey {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
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
                    map.serialize_entry(&LuaMapKey::to_string(k), v)?;
                }
                map.end()
            }
        }
    }
}
