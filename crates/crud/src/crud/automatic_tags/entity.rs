// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Automatic tags for entities
//!

use crate::crud::automatic_tags::helper_value_to_value;
use bool_tag_expr::Tag;
use std::collections::HashMap;

/// Create mappings from one tag to another.  Mappings should be as direct as
/// possible.  For example, one should not map "physicist" -> "person", but
/// instead map "physicist" -> "scientist", and "scientist" -> "person".
///
/// One must be careful to to map incorrectly.  For example, "british" should
/// not be mapped to "person", because of course something not a person (such as
/// a boat) can be tagged with "british"
pub(super) fn default_entity_tags() -> HashMap<Tag, Tag> {
    HashMap::from([
        // to <-- from
        //
        value_value!("artist" <-- "author"),
        value_value!("artist" <-- "musician"),
        value_value!("artist" <-- "painter"),
        value_value!("artist" <-- "poet"),
        //
        value_value!("artwork" <-- "novel"),
        value_value!("artwork" <-- "painting"),
        value_value!("artwork" <-- "play"),
        value_value!("artwork" <-- "poem"),
        //
        value_value!("monarch" <-- "king"),
        value_value!("monarch" <-- "queen"),
        //
        value_value!("musician" <-- "guitarist"),
        value_value!("musician" <-- "pianist"),
        value_value!("musician" <-- "singer"),
        value_value!("musician" <-- "violinist"),
        //
        value_value!("person" <-- "actor"),
        value_value!("person" <-- "artist"),
        value_value!("person" <-- "emperor"),
        value_value!("person" <-- "empress"),
        value_value!("person" <-- "historian"),
        value_value!("person" <-- "lawyer"),
        value_value!("person" <-- "linguist"),
        value_value!("person" <-- "mathematician"),
        value_value!("person" <-- "monarch"),
        value_value!("person" <-- "nobel-laureate"),
        value_value!("person" <-- "philosopher"),
        value_value!("person" <-- "philologist"),
        value_value!("person" <-- "playwright"),
        value_value!("person" <-- "politician"),
        value_value!("person" <-- "sailor"),
        value_value!("person" <-- "saint"),
        value_value!("person" <-- "scholar"),
        value_value!("person" <-- "scientist"),
        value_value!("person" <-- "writer"),
        //
        value_value!("politician" <-- "member-of-parliament"),
        value_value!("politician" <-- "president"),
        value_value!("politician" <-- "prime-minister"),
        value_value!("politician" <-- "senator"),
        value_value!("politician" <-- "vice-president"),
        //
        value_value!("scientist" <-- "biologist"),
        value_value!("scientist" <-- "chemist"),
        value_value!("scientist" <-- "physicist"),
        //
        value_value!("writer" <-- "author"),
        value_value!("writer" <-- "novellist"),
        value_value!("writer" <-- "poet"),
        value_value!("writer" <-- "journalist"),
    ])
}

#[cfg(test)]
mod test {
    use super::*;

    /// Ensure the default tag mappings are valid
    #[test]
    fn default_are_valid() {
        default_entity_tags();
    }
}
