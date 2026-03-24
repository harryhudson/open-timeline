// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! HTML quiz generation for the which date game
//!

use super::*;
use crate::html::{Html, HtmlQuiz};

impl HtmlQuiz for WhichDateGame {
    fn generate_html_quiz(&mut self, _question_count: usize) -> Result<(Html, Html), ()> {
        unimplemented!()
    }
}
