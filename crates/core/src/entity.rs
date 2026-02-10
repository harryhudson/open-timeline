// SPDX-License-Identifier: MIT

//!
//! The OpenTimeline entity type
//!

use crate::{Date, Day, HasIdAndName, Month, Name, OpenTimelineId, Year};
use bool_tag_expr::{BoolTagExpr, Node, Tag, Tags};
use serde::{Deserialize, Deserializer, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

// TODO: improve (add more fine grain variants)?
/// Errors that can arise in relation to an [`Entity`]
#[derive(Error, Debug)]
pub enum EntityError {
    #[error("The entity dates are invalid")]
    Dates,
}

/// The OpenTimeline [`Entity`] type
#[derive(Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity {
    /// The entity's ID
    id: Option<OpenTimelineId>,

    /// The entity's name
    name: Name,

    /// When did the entity begin/start
    start: Date,

    /// When did the entity end/finish (if it has)
    end: Option<Date>,

    /// Tags for the entity
    tags: Option<Tags>,
}

// TODO: write a derive macro to derive Ord only from the ID for use with
// all types?
// Ord using just the Entity ID (Date does not, and can not, have a full ordering)
impl Ord for Entity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

// Just use the full Ord
impl PartialOrd for Entity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Entity {
    /// Create a valid OpenTimeline [`Entity`] if it is possible to do so with
    /// the values passed in
    pub fn from(
        id: Option<OpenTimelineId>,
        name: Name,
        start: Date,
        end: Option<Date>,
        tags: Option<Tags>,
    ) -> Result<Entity, EntityError> {
        let entity = Entity {
            id,
            name,
            start,
            end,
            tags,
        };

        if entity.has_valid_dates() {
            Ok(entity)
        } else {
            Err(EntityError::Dates)
        }
    }

    /// Clear the [`Entity`]'s ID
    pub fn clear_id(&mut self) {
        self.id = None;
    }

    /// Whether the entity has valid dates
    fn has_valid_dates(&self) -> bool {
        if let Some(end) = &self.end {
            if end < &self.start {
                return false;
            }
        }
        true
    }

    /// Get the entity's [`Tags`]
    pub fn tags(&self) -> &Option<Tags> {
        &self.tags
    }

    /// Set the entity's [`Tags`]
    pub fn set_tags(&mut self, tags: Tags) {
        self.tags = (!tags.is_empty()).then_some(tags);
    }

    /// Clear the entity's [`Tags`] and set to `None`
    pub fn clear_tags(&mut self) {
        self.tags = None;
    }

    /// Add a tag to the entity
    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.get_or_insert_with(Tags::new).insert(tag);
    }

    /// Remove a tag from the entity
    pub fn remove_tag(&mut self, tag: &Tag) {
        if let Some(tags) = self.tags.as_mut() {
            tags.remove(tag);
            if tags.is_empty() {
                self.tags = None
            }
        }
    }

    /// Get the entity's start [`Date`]
    pub fn start(&self) -> Date {
        self.start
    }

    /// Set the entity's start [`Date`] if it'll be valid
    pub fn set_start(&mut self, start: Date) -> Result<(), EntityError> {
        let mut tmp_entity = self.clone();
        tmp_entity.start = start;
        if !tmp_entity.has_valid_dates() {
            return Err(EntityError::Dates);
        }
        self.start = start;
        Ok(())
    }

    /// Get the entity's end [`Date`]
    pub fn end(&self) -> Option<Date> {
        self.end
    }

    /// Set the entity's end [`Date`] if it'll be valid
    pub fn set_end(&mut self, end: Date) -> Result<(), EntityError> {
        let mut tmp_entity = self.clone();
        tmp_entity.end = Some(end);
        if !tmp_entity.has_valid_dates() {
            return Err(EntityError::Dates);
        }
        self.end = Some(end);
        Ok(())
    }

    /// Check if the entity's end year is set
    pub fn end_year_is_set(&self) -> bool {
        self.end_year().is_some()
    }

    /// Check if the entity's end year is set
    pub fn end_year(&self) -> Option<Year> {
        self.end.map(|date| date.year())
    }

    /// Check if the entity's end month is set
    pub fn end_month(&self) -> Option<Month> {
        self.end.and_then(|date| date.month())
    }

    /// Check if the entity's end day is set
    pub fn end_day(&self) -> Option<Day> {
        self.end.and_then(|date| date.day())
    }

    /// Check if the entity's start year is set
    pub fn start_year(&self) -> Year {
        self.start.year()
    }

    /// Check if the entity's start month is set
    pub fn start_month(&self) -> Option<Month> {
        self.start.month()
    }

    /// Check if the entity's start day is set
    pub fn start_day(&self) -> Option<Day> {
        self.start.day()
    }

    /// Whether the entity in question matches the boolean tag expression.  This
    /// can be used to filter a list of entities by a boolean tag expression.
    pub fn matches_bool_tag_expr(&self, bool_tag_expr: &BoolTagExpr) -> bool {
        let Some(tags) = self.tags() else {
            return false;
        };

        // TODO: move into bool-tag-expr crate
        /// Evaluate a `BooleanTagExpr` tree against a list of `Tags`
        fn evaluate_in_one(expr: Node, tags: &Tags) -> bool {
            match expr {
                Node::And(l, r) => evaluate_in_one(*l, tags) && evaluate_in_one(*r, tags),
                Node::Or(l, r) => evaluate_in_one(*l, tags) || evaluate_in_one(*r, tags),
                Node::Not(e) => !evaluate_in_one(*e, tags),
                Node::Tag(tag) => tags.contains(&tag),
                Node::Bool(_) => panic!(),
            }
        }

        // TODO: add a .as_node()/.node() method to bool-tag-expr crate so no cloning
        evaluate_in_one(bool_tag_expr.clone().into_node(), tags)
    }
}

impl HasIdAndName for Entity {
    fn id(&self) -> Option<OpenTimelineId> {
        self.id
    }

    fn set_id(&mut self, id: OpenTimelineId) {
        self.id = Some(id)
    }

    fn name(&self) -> &Name {
        &self.name
    }

    fn set_name(&mut self, name: Name) {
        self.name = name
    }
}

/// Used only by the custom deserialiser (to make it simpler)
#[derive(Deserialize, Debug)]
pub struct RawEndDate {
    day: Option<i64>,
    month: Option<i64>,
    year: Option<i64>,
}

/// Used only by the custom deserialiser (to make it simpler)
#[derive(Deserialize, Debug)]
struct RawEntity {
    id: Option<OpenTimelineId>,
    name: Name,
    start: Date,
    end: Option<RawEndDate>,
    tags: Option<Tags>,
}

impl<'de> Deserialize<'de> for Entity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // TODO: look into serde Visitors (and do without RawEntity)
        let raw_entity = RawEntity::deserialize(deserializer)?;

        // Deal with the incoming JSON having:
        // "end":{"year":null,"month":null,"day":null}
        // ie end isn't null but should be
        let end = match raw_entity.end {
            None => None,
            Some(end) => {
                if end.day.is_none() && end.month.is_none() && end.year.is_none() {
                    None
                } else if end.year.is_none() {
                    // i.e. year is None, but day OR month are Some
                    let err_msg = String::from(
                        "End year is invalid (day and/or month is set, but year isn't",
                    );
                    return Err(serde::de::Error::custom(err_msg));
                } else {
                    match Date::from(end.day, end.month, end.year.unwrap()) {
                        Ok(end) => Some(end),
                        Err(_) => {
                            // TODO: improve
                            let err_msg = String::from("End year is invalid");
                            return Err(serde::de::Error::custom(err_msg));
                        }
                    }
                }
            }
        };

        Entity::from(
            raw_entity.id,
            raw_entity.name,
            raw_entity.start,
            end,
            raw_entity.tags,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bool_tag_expr::TagValue;
    use open_timeline_macros::{day, month, year};
    use std::{
        collections::BTreeSet,
        fs::{self, File},
        io::{self, BufRead},
        path::PathBuf,
    };

    const KNOWN_UUIDV4: &str = "6474cd74-244d-449b-a3d1-3a74019ec6f5";

    fn valid_entity() -> Entity {
        Entity::from(
            Some(OpenTimelineId::from(KNOWN_UUIDV4).unwrap()),
            Name::from("Noam").unwrap(),
            Date::from(None, None, 1111).unwrap(),
            Some(Date::from(None, None, 2222).unwrap()),
            Some(Tags::new()),
        )
        .unwrap()
    }

    // TODO (more checks - for tags (if empty set to None instead))
    #[test]
    fn from() {
        let entity = Entity::from(
            Some(OpenTimelineId::new()),
            Name::from("Noam").unwrap(),
            Date::from(None, None, 1111).unwrap(),
            Some(Date::from(None, None, 2222).unwrap()),
            Some(Tags::new()),
        );
        assert!(entity.is_ok());
    }

    #[test]
    fn name_getters_and_setters() {
        // Get a valid entity
        let mut entity = valid_entity();

        // Check the name getter
        assert_eq!(entity.name(), &Name::from("Noam").unwrap());

        // Use the name setter
        entity.set_name(Name::from("Alan").unwrap());

        // Check the name setter
        assert_eq!(entity.name(), &Name::from("Alan").unwrap());
    }

    #[test]
    fn id_getters_and_setters() {
        // Get a valid entity
        let mut entity = valid_entity();

        // Check the ID getter
        assert_eq!(
            entity.id(),
            Some(OpenTimelineId::from(KNOWN_UUIDV4).unwrap())
        );

        // Get known ID
        let id = OpenTimelineId::new();

        // Use the ID setter
        entity.set_id(id);

        // Check the ID setter
        assert_eq!(entity.id(), Some(id));

        // Use the ID clearer
        entity.clear_id();

        // Check the ID clearer
        assert!(entity.id().is_none());
    }

    #[test]
    fn date_getters_and_setters() {
        // Get a valid entity
        let mut entity = valid_entity();

        let start = entity.start();
        let end = entity.end().unwrap();

        // Check the start date setter

        // Start after end
        assert!(
            entity
                .set_start(Date::from(Some(1), Some(2), 3333).unwrap())
                .is_err()
        );
        assert_eq!(entity.start(), start);

        // Start before end
        assert!(
            entity
                .set_start(Date::from(Some(1), Some(2), 3).unwrap())
                .is_ok()
        );
        assert_ne!(entity.start(), start);

        // Check the end date setter

        // End before start
        assert!(
            entity
                .set_end(Date::from(Some(4), Some(5), -6543).unwrap())
                .is_err()
        );
        assert_eq!(entity.end().unwrap(), end);

        // End after start
        assert!(
            entity
                .set_end(Date::from(Some(4), Some(5), 6).unwrap())
                .is_ok()
        );
        assert_ne!(entity.end().unwrap(), end);

        // Check the date getters
        assert_eq!(entity.start_year(), year!(3));
        assert_eq!(entity.start_month(), Some(month!(2)));
        assert_eq!(entity.start_day(), Some(day!(1)));

        assert_eq!(entity.end_year(), Some(year!(6)));
        assert_eq!(entity.end_month(), Some(month!(5)));
        assert_eq!(entity.end_day(), Some(day!(4)));
    }

    #[test]
    fn deserialisation() {
        let path_to_test_data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data");

        // Check the valid JSON entities can be parsed
        for entry in fs::read_dir(path_to_test_data.join("entities/valid")).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "jsonc") {
                let json_content = load_jsonc_strip_leading_comment_lines(&path);
                println!("Reading file: {:?}", path);
                println!("{}", json_content);
                let entities: Result<Vec<Entity>, serde_json::Error> =
                    serde_json::from_str(&json_content);
                assert!(entities.is_ok())
            }
        }

        // Check the invalid JSON entities cannot be parsed
        for entry in fs::read_dir(path_to_test_data.join("entities/invalid")).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "jsonc") {
                println!("Reading file: {:?}", path);
                let json_content = load_jsonc_strip_leading_comment_lines(&path);
                println!("{}", json_content);
                let entities: Result<Vec<Entity>, serde_json::Error> =
                    serde_json::from_str(&json_content);
                assert!(entities.is_err())
            }
        }
    }

    pub fn load_jsonc_strip_leading_comment_lines(path: &PathBuf) -> String {
        // Open the file for reading
        let file = File::open(path).unwrap();
        let reader = io::BufReader::new(file);

        // Holds the JSON as it's collected
        let mut json_content = String::new();

        // Collect all lines that don't begin with "//"
        for line in reader.lines() {
            let line = line.unwrap();
            if !line.starts_with("//") {
                json_content.push_str(&line);
                json_content.push('\n');
            }
        }

        // Return the JSON now that the comment(s) at the top of the file have
        // been removed
        json_content
    }

    #[test]
    fn matches_bool_tag_expr() -> Result<(), Box<dyn std::error::Error>> {
        //
        // 1. expr with 1 tag value
        //

        // Create bool expr
        let bool_tag_expr = BoolTagExpr::from("a")?;

        // Add tag to entity
        let tags = Tags::from([Tag::from(None, TagValue::from(&"a")?)]);
        let mut entity_a = valid_entity();
        entity_a.tags = Some(tags);

        // Should match
        assert!(entity_a.matches_bool_tag_expr(&bool_tag_expr));

        //
        // 2. expr with 2 tag values
        //

        // Create bool expr
        let bool_tag_expr = BoolTagExpr::from("a & b")?;

        // Add only 1 tag to the entity
        let tags = Tags::from([Tag::from(None, TagValue::from(&"a")?)]);
        let mut entity_a = valid_entity();
        entity_a.tags = Some(tags);

        // Shouldn't match
        assert!(!entity_a.matches_bool_tag_expr(&bool_tag_expr));

        // Add 2nd tag to entity
        entity_a
            .tags
            .get_or_insert_with(BTreeSet::new)
            .insert(Tag::from(None, TagValue::from(&"b")?));

        // Should match
        assert!(entity_a.matches_bool_tag_expr(&bool_tag_expr));

        //
        // 2. expr with 2 tag values, 1 is NOT (reverse expected results of test 2)
        //

        // Create bool expr (note use of `!`)
        let bool_tag_expr = BoolTagExpr::from("a & !b")?;

        // Add only 1 tag to the entity
        let tags = Tags::from([Tag::from(None, TagValue::from(&"a")?)]);
        let mut entity_a = valid_entity();
        entity_a.tags = Some(tags);

        // Should match this time (doesn't have tag `b`)
        assert!(entity_a.matches_bool_tag_expr(&bool_tag_expr));

        // Add 2nd tag to entity
        entity_a
            .tags
            .get_or_insert_with(BTreeSet::new)
            .insert(Tag::from(None, TagValue::from(&"b")?));

        // Shouldn't match
        assert!(!entity_a.matches_bool_tag_expr(&bool_tag_expr));

        //
        // 3. Advanced expr
        //

        // Create advanced bool expr
        let bool_tag_expr = BoolTagExpr::from("(a | b & c) & !(d & e)")?;

        // Add only tag `a` to the entity (should match)
        let tags = Tags::from([Tag::from(None, TagValue::from(&"a")?)]);
        let mut entity_a = valid_entity();
        entity_a.tags = Some(tags);
        assert!(entity_a.matches_bool_tag_expr(&bool_tag_expr));

        // Add only tag `a` to the entity (shouldn't match)
        let tags = Tags::from([
            Tag::from(None, TagValue::from(&"a")?),
            Tag::from(None, TagValue::from(&"d")?),
            Tag::from(None, TagValue::from(&"e")?),
        ]);
        entity_a.tags = Some(tags);
        assert!(!entity_a.matches_bool_tag_expr(&bool_tag_expr));

        // Add only tag `a` to the entity (shouldn't match)
        let tags = Tags::from([
            Tag::from(None, TagValue::from(&"a")?),
            Tag::from(None, TagValue::from(&"b")?),
            Tag::from(None, TagValue::from(&"c")?),
            Tag::from(None, TagValue::from(&"d")?),
            Tag::from(None, TagValue::from(&"superfluous")?),
        ]);
        entity_a.tags = Some(tags);
        assert!(entity_a.matches_bool_tag_expr(&bool_tag_expr));

        Ok(())
    }
}
