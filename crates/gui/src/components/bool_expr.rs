// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Everything needed to handle 1 boolean expression
//!

use crate::common::ToOpenTimelineType;
use bool_tag_expr::BoolTagExpr;
use open_timeline_gui_core::BooleanExpressionGui;

impl ToOpenTimelineType<BoolTagExpr> for BooleanExpressionGui {
    fn to_opentimeline_type(&self) -> BoolTagExpr {
        self.try_into().unwrap()
    }
}
