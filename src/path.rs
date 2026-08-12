use crate::error::{HdError, Result};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
const HARDENED: u32 = 1 << 31;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChildIndex(pub u32);
impl ChildIndex {
    pub fn normal(n: u32) -> Result<Self> {
        if n >= HARDENED {
            Err(HdError::InvalidChildIndex(n.to_string()))
        } else {
            Ok(Self(n))
        }
    }
    pub fn hardened(n: u32) -> Result<Self> {
        if n >= HARDENED {
            Err(HdError::InvalidChildIndex(n.to_string()))
        } else {
            Ok(Self(n | HARDENED))
        }
    }
    pub fn is_hardened(self) -> bool {
        self.0 & HARDENED != 0
    }
    pub fn number(self) -> u32 {
        self.0 & !HARDENED
    }
    pub fn raw(self) -> u32 {
        self.0
    }
}
impl FromStr for ChildIndex {
    type Err = HdError;
    fn from_str(s: &str) -> Result<Self> {
        let hard = s.ends_with('\'') || s.ends_with('h') || s.ends_with('H');
        let nstr = if hard { &s[..s.len() - 1] } else { s };
        let n: u32 = nstr
            .parse()
            .map_err(|_| HdError::InvalidChildIndex(s.into()))?;
        if hard {
            Self::hardened(n)
        } else {
            Self::normal(n)
        }
    }
}

impl fmt::Display for ChildIndex {
    // fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //     write!(f, "{}", self.0)
    // }
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivationPath(pub Vec<ChildIndex>);
impl FromStr for DerivationPath {
    type Err = HdError;
    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim_matches('/');
        if s.is_empty() {
            return Ok(Self(vec![]));
        }
        let mut parts = s.split('/').collect::<Vec<_>>();
        if parts.len() >= 2
            && parts[0].parse::<ChildIndex>().is_err()
            && parts[1].parse::<ChildIndex>().is_err()
        {
            parts.drain(0..2);
        }
        parts
            .into_iter()
            .map(str::parse)
            .collect::<Result<Vec<_>>>()
            .map(Self)
    }
}

impl fmt::Display for DerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("/")
        )
    }
}
