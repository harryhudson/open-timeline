// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Automatic tags - allows one to specify that if an entity or timeline has tag
//! X that they should also have tags Y & Z.
//!

/// Helper macro to map tag values to tag values (string literals)
macro_rules! value_value {
    // Usage: `value_value!("from" --> "to");`
    ($from:literal --> $to:literal) => {
        helper_value_to_value($from, $to)
    };

    // Usage: `value_value!("to" <-- "from");`
    ($to:literal <-- $from:literal) => {
        helper_value_to_value($from, $to)
    };
}

mod entity;
mod timeline;

use crate::crud::automatic_tags::{entity::default_entity_tags, timeline::default_timeline_tags};
use bool_tag_expr::{Tag, TagValue, Tags};
use open_timeline_core::{Entity, TimelineEdit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Whether the tags have be altered or not
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum TagsAltered {
    Unaltered,
    Altered,
}

/// Automatically add tags to entity & timelines using their existing tags. For
/// example an entity tagged with "king" can also be tagged with "person"
/// automatically if it doesn't already have that tag
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AutomticTag {
    /// Entity tag maps - if an entity has a key tag it will be given the
    /// corresponding value tag
    entity_tags_map: HashMap<Tag, Tag>,

    /// Timeline tag maps - if a timeline has a key tag it will be given the
    /// corresponding value tag
    timeline_tags_map: HashMap<Tag, Tag>,
}

impl AutomticTag {
    /// Use the entity tag mapping and return the (possible augmented) tags
    pub fn map_entity_tags(&self, entity: &mut Entity) -> TagsAltered {
        if let Some(tags) = entity.tags_mut() {
            self.map_tags(tags, &self.entity_tags_map)
        } else {
            TagsAltered::Unaltered
        }
    }

    /// Use the timeline tag mapping and return the (possible augmented) tags
    pub fn map_timeline_tags(&self, timeline: &mut TimelineEdit) -> TagsAltered {
        if let Some(tags) = timeline.tags_mut() {
            self.map_tags(tags, &self.entity_tags_map)
        } else {
            TagsAltered::Unaltered
        }
    }

    /// Helper to apply the mappings
    fn map_tags(&self, tags: &mut Tags, map: &HashMap<Tag, Tag>) -> TagsAltered {
        let mut tags_altered = TagsAltered::Unaltered;
        loop {
            // Track whether the tags are changed
            let mut tags_changed = false;

            // Loop over the existing tags
            for original in tags.clone().into_iter() {
                // If the existing tag has a mapping, insert the new key to the
                // original keys and note whether the original tags already
                // included the new tag
                if let Some(new_tag) = map.get(&original) {
                    if tags.insert(new_tag.clone()) {
                        tags_changed = true;
                        tags_altered = TagsAltered::Altered;
                    }
                }
            }

            // If the tags have changed, loop again incase any new mapping
            // conditions are met
            if !tags_changed {
                break;
            }
        }

        // Return whether the tags have been altered
        tags_altered
    }
}

impl Default for AutomticTag {
    fn default() -> Self {
        Self {
            entity_tags_map: default_entity_tags(),
            timeline_tags_map: default_timeline_tags(),
        }
    }
}

/// Map a tag with only a value to another tag with only a value
fn helper_value_to_value(existing: &str, new: &str) -> (Tag, Tag) {
    (
        Tag::from(None, TagValue::from(&existing).unwrap()),
        Tag::from(None, TagValue::from(&new).unwrap()),
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use bool_tag_expr::Tags;
    use open_timeline_core::{Date, Name};

    /// Check the mappings are applied corrected
    #[test]
    fn mapping() {
        // Create tags for an entity
        let tags = Tags::from([Tag::from(None, TagValue::from("king").unwrap())]);
        let original_tags = tags.clone();
        let mut entity = Entity::from(
            None,
            Name::from("bob").unwrap(),
            Date::from(None, None, 1234).unwrap(),
            None,
            Some(tags),
        )
        .unwrap();

        // Run the tag mappings
        let tags_altered = AutomticTag::default().map_entity_tags(&mut entity);

        // Check the response
        assert_eq!(tags_altered, TagsAltered::Altered);

        // Check the original tags and the augmented tags do not match
        assert_ne!(original_tags, entity.tags().clone().unwrap());

        // Check the new tags collection has more than 2 tags
        assert!(entity.tags().as_ref().unwrap().len() >= 2);

        // Check the new tags collection has the tag "person"
        assert!(
            entity
                .tags()
                .as_ref()
                .unwrap()
                .contains(&Tag::from(None, TagValue::from("person").unwrap()))
        );
    }

    /// Check the macro
    #[test]
    fn check_macro() {
        let first = HashMap::from([helper_value_to_value("king", "person")]);
        let second = HashMap::from([value_value!("person" <-- "king")]);
        let third = HashMap::from([value_value!("king" --> "person")]);

        // Check the 3 are all equal
        assert_eq!(first, second);
        assert_eq!(first, third);
    }
}
