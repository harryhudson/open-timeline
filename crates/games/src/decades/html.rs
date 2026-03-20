// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! HTML quiz generation
//!

use super::*;
use crate::html::{Html, HtmlQuiz};
use open_timeline_core::HasIdAndName;
use rand::seq::SliceRandom;

impl HtmlQuiz for DecadesGame {
    fn generate_html_quiz(&mut self, question_count: usize) -> Result<(Html, Html), ()> {
        // Get Qs
        let mut questions = Vec::new();
        let mut rng = rand::thread_rng();
        loop {
            let entity = self.entity_pool.partial_shuffle(&mut rng, 1).0[0].clone();
            if let Ok(question) = generate_text_question(entity) {
                questions.push(question);
            }
            if questions.len() == question_count {
                break;
            }
        }

        // Begin HTML docs
        let mut html_quiz = Vec::new();
        let mut html_answers = Vec::new();
        html_quiz.push(Html::html_opening_quiz_doc(
            "Quiz Questions",
            vec!["", "Questions", "", "", ""],
        ));
        html_answers.push(Html::html_opening_quiz_doc(
            "Quiz Answers",
            vec!["", "Questions", "", "", ""],
        ));

        // Create HTML tables for Qs and As
        for (i, question) in questions.iter().enumerate() {
            html_quiz.push(Html::quiz_table_row(vec![
                &i.to_string(),
                question.entity.name().as_str(),
                question.options[0]
                    .to_html_question(|date| date.to_string())
                    .str(),
                question.options[1]
                    .to_html_question(|date| date.to_string())
                    .str(),
                question.options[2]
                    .to_html_question(|date| date.to_string())
                    .str(),
            ]));

            html_answers.push(Html::quiz_table_row(vec![
                &i.to_string(),
                question.entity.name().as_str(),
                question.options[0]
                    .to_html_answer(|date| date.to_string())
                    .str(),
                question.options[1]
                    .to_html_answer(|date| date.to_string())
                    .str(),
                question.options[2]
                    .to_html_answer(|date| date.to_string())
                    .str(),
            ]));
        }

        // Finish HTML docs
        html_quiz.push(Html::quiz_html_doc_finish());
        html_answers.push(Html::quiz_html_doc_finish());

        // Return the HTML
        Ok((Html::from_vec(html_quiz), Html::from_vec(html_answers)))
    }
}
