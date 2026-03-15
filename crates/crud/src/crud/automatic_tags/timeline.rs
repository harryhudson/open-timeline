// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Automatic tags for timelines
//!

use bool_tag_expr::Tag;
use std::collections::HashMap;

pub(super) fn default_timeline_tags() -> HashMap<Tag, Tag> {
    HashMap::from([
        // None
    ])
}

#[cfg(test)]
mod test {
    use super::*;

    /// Ensure the default tag mappings are valid
    #[test]
    fn default_are_valid() {
        default_timeline_tags();
    }
}
