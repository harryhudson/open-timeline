// SPDX-License-Identifier: MIT

//!
//! The OpenTimeline name type
//!

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

// TODO: should these <Type>Error enums have (de)serialising errors too?
/// Errors that can arise in relation to a [`Name`]
#[derive(Error, Debug, Clone)]
pub enum NameError {
    #[error("Name cannot be empty")]
    Empty,
}

// TODO: consider impl Deref to str so can be used where &str is expected
/// The OpenTimeline [`Name`] type.  The value can be any string apart from one
/// which when trimmed of trailing and leading whitespace is empty.
#[derive(derive_more::Display, Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct Name(String);

impl Name {
    /// Create and initialise a new name if it will be valid
    pub fn from<S: ToString>(name: S) -> Result<Self, NameError> {
        let name = name.to_string();
        if name.trim().is_empty() {
            Err(NameError::Empty)
        } else {
            Ok(Name(name.trim().to_string()))
        }
    }

    /// Get the underlying `&str`
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Name::from(string).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from() {
        assert!(Name::from("").is_err());
        assert!(Name::from("  ").is_err());
        let ok_1 = Name::from("Pass").unwrap();
        let ok_2 = Name::from(" Pass ").unwrap();
        assert_eq!(ok_1, ok_2)
    }
}
