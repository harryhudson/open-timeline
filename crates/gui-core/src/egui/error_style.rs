// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Those things used across the OpenTimeline GUI crate
//!

use crate::{Valid, ValidityAsynchronous};
use eframe::egui::{Color32, Context, Ui};

/// Implementing types can display the validity of the data they hold.
///
/// Implementing types do not need to implement their own funcitonality; they
/// need only decalre that they implement it.  The default implementation uses
/// a private global.
pub trait ErrorStyle: Valid {
    fn set_validity_styling(&mut self, _ctx: &Context, ui: &mut Ui) {
        if let ValidityAsynchronous::Invalid(_) = self.validity() {
            let visuals = ui.visuals_mut();
            visuals.override_text_color = Some(Color32::WHITE);
            visuals.extreme_bg_color = Color32::LIGHT_RED;
            visuals.text_edit_bg_color = Some(Color32::LIGHT_RED);
        }
    }
}
