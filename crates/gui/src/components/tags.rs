// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! A couple of OpenTimeline impls for [`TagsGui`] and [`TagGui`]
//!

use crate::common::ToOpenTimelineType;
use bool_tag_expr::{Tag, Tags};
use open_timeline_gui_core::{TagGui, TagsGui};

// TODO: EntityTag and TimelineTag??

impl ToOpenTimelineType<Option<Tags>> for TagsGui {
    fn to_opentimeline_type(&self) -> Option<Tags> {
        self.try_into().unwrap()
    }
}

impl ToOpenTimelineType<Tag> for TagGui {
    // TODO: reuse validation
    fn to_opentimeline_type(&self) -> Tag {
        self.try_into().unwrap()
    }
}
