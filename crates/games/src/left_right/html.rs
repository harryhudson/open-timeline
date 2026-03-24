// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! HTML quiz generation for the left-right game
//!

use super::*;
use crate::html::{Html, HtmlQuiz};

impl HtmlQuiz for LeftRightGame {
    fn generate_html_quiz(&mut self, _question_count: usize) -> Result<(Html, Html), ()> {
        unimplemented!()
    }
}
