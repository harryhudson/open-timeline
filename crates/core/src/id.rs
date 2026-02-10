// SPDX-License-Identifier: MIT

//!
//! Functions for ID management (create a globally unique one, or check if it
//! already exists)
//!

use uuid::Uuid;

/// The OpenTimeline ID type is a UUIDv4
#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(derive_more::Display, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct OpenTimelineId(Uuid);

impl OpenTimelineId {
    /// Create a new `OpenTimelineId`
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create an ID from a string if the string is a valid ID
    pub fn from<S: ToString>(string: S) -> Result<Self, uuid::Error> {
        let string = string.to_string();
        Ok(Self(Uuid::parse_str(&string)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_id_deserialization() {
        let uuid_str = r#""550e8400-e29b-41d4-a716-446655440000""#;
        let id: OpenTimelineId = serde_json::from_str(uuid_str).expect("Failed to deserialize");
        assert_eq!(
            id,
            OpenTimelineId(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap())
        );

        // Remove a "0" from the end (should fail because not valid)
        let uuid_str = r#""550e8400-e29b-41d4-a716-44665544000""#;
        assert!(serde_json::from_str::<OpenTimelineId>(uuid_str).is_err());
    }

    #[test]
    fn test_id_serialization() {
        let id = OpenTimelineId(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""550e8400-e29b-41d4-a716-446655440000""#);
        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }
}
