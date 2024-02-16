use std::collections::HashMap;
use std::fmt;

#[macro_export]
macro_rules! kv {
    ($($k:expr =>  $v:expr;)*) => {
        $crate::core::KV { kv: std::collections::HashMap::from([ $(($k, $v.into())),* ]) }
    };
}

#[derive(Clone, PartialEq, Debug)]
pub enum KvValue {
    /// Floating point values
    Float(f64),
    /// Signed integers
    Int(i64),
    /// Unsigned integers
    Uint(u64),
    /// Boolean values
    Bool(bool),
    /// Strings
    Str(String),
}

impl From<f64> for KvValue {
    fn from(x: f64) -> Self {
        Self::Float(x)
    }
}

impl From<f32> for KvValue {
    fn from(x: f32) -> Self {
        Self::Float(f64::from(x))
    }
}

impl From<i64> for KvValue {
    fn from(x: i64) -> Self {
        Self::Int(x)
    }
}

impl From<u64> for KvValue {
    fn from(x: u64) -> Self {
        Self::Uint(x)
    }
}

impl From<i32> for KvValue {
    fn from(x: i32) -> Self {
        Self::Int(i64::from(x))
    }
}

impl From<u32> for KvValue {
    fn from(x: u32) -> Self {
        Self::Uint(u64::from(x))
    }
}

impl From<bool> for KvValue {
    fn from(x: bool) -> Self {
        Self::Bool(x)
    }
}

impl From<String> for KvValue {
    fn from(x: String) -> Self {
        Self::Str(x)
    }
}

impl<'a> From<&'a str> for KvValue {
    fn from(x: &'a str) -> Self {
        Self::Str(x.to_string())
    }
}

impl KvValue {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Float(_) => "Float",
            Self::Int(_) => "Int",
            Self::Uint(_) => "Uint",
            Self::Bool(_) => "Bool",
            Self::Str(_) => "Str",
        }
    }
}

impl fmt::Display for KvValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Float(x) => write!(f, "{x}")?,
            Self::Int(x) => write!(f, "{x}")?,
            Self::Uint(x) => write!(f, "{x}")?,
            Self::Bool(x) => write!(f, "{x}")?,
            Self::Str(x) => write!(f, "{x}")?,
        };
        Ok(())
    }
}

pub struct KV {
    /// The actual key value storage
    pub kv: HashMap<&'static str, KvValue>,
}

impl KV {
    pub fn new() -> Self {
        Self { kv: HashMap::new() }
    }

    pub fn insert(&mut self, key: &'static str, val: KvValue) -> &mut Self {
        self.kv.insert(key, val);
        self
    }

    pub fn get(&self, key: &'static str) -> Option<&KvValue> {
        self.kv.get(key)
    }

    pub fn keys(&self) -> Vec<(&'static str, &'static str)> {
        self.kv.iter().map(|(&k, v)| (k, v.kind())).collect()
    }

    #[must_use]
    pub fn merge(mut self, other: Self) -> Self {
        self.kv.extend(other.kv);
        self
    }
}

impl fmt::Debug for KV {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{self}")?;
        Ok(())
    }
}
impl fmt::Display for KV {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "KV")?;
        for (key, val) in &self.kv {
            writeln!(f, "   {key}: {val}")?;
        }
        Ok(())
    }
}
