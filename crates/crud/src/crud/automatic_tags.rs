// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Automatic tags - allows one to specify that if an entity or timeline has tag
//! X that they should also have tags Y & Z.
//!

use bool_tag_expr::{Tag, TagValue, Tags};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represent choice of entity or timeline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntityOrTimeline {
    Entity,
    Timeline,
}

/// Automatically add tags to entity & timelines using their existing tags. For
/// example an entity tagged with "king" can also be tagged with "person"
/// automatically if it doesn't already have that tag
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AutomticTag {
    /// Entity tag maps - if an entity has a key tag it will be given the
    /// corresponding value tag
    entity_tags: HashMap<Tag, Tag>,

    /// Timeline tag maps - if a timeline has a key tag it will be given the
    /// corresponding value tag
    timeline_tags: HashMap<Tag, Tag>,
}

impl AutomticTag {
    /// Use the entity tag mapping and return the (possible augmented) tags
    pub fn map_entity_tags(&self, tags: Tags) -> Tags {
        self.map_tags(tags, EntityOrTimeline::Entity)
    }

    /// Use the timeline tag mapping and return the (possible augmented) tags
    pub fn map_timeline_tags(&self, tags: Tags) -> Tags {
        self.map_tags(tags, EntityOrTimeline::Timeline)
    }

    /// Helper to apply the mappings
    fn map_tags(&self, mut tags: Tags, entity_or_timeline: EntityOrTimeline) -> Tags {
        // Get the correct map
        let map = match entity_or_timeline {
            EntityOrTimeline::Entity => &self.entity_tags,
            EntityOrTimeline::Timeline => &self.timeline_tags,
        };

        // Begin the loop
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
                    }
                }
            }

            // If the tags have changed, loop again incase any new mapping
            // conditions are met
            if !tags_changed {
                break;
            }
        }

        // Return the (possibly augmented) tags
        tags
    }
}

impl Default for AutomticTag {
    fn default() -> Self {
        Self {
            entity_tags: HashMap::from([
                helper_value_to_value("scientist", "person"),
                helper_value_to_value("king", "person"),
                helper_value_to_value("emperor", "person"),
            ]),
            timeline_tags: HashMap::from([]),
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

    /// Ensure the default tag mappings are valid
    #[test]
    fn default_are_valid() {
        AutomticTag::default();
    }

    /// Ensure the default tag mappings are valid
    #[test]
    fn mapping() {
        // Create tags for an entity
        let tags = Tags::from([Tag::from(None, TagValue::from(&"king").unwrap())]);

        // Get the tag mappings
        let tag_mappings = AutomticTag::default();

        // Run the tag mappings
        let new_tags = tag_mappings.map_entity_tags(tags.clone());

        // Check the original tags and the new tags do not match
        assert_ne!(tags, new_tags);

        // Check the new tags collection has 2 tags
        assert_eq!(2, new_tags.len());

        // Check the new tags collection has the tag "person"
        assert!(new_tags.contains(&Tag::from(None, TagValue::from(&"person").unwrap())));
    }
}
