// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! HTML quiz generation for the were they alive when game
//!

use super::*;
use crate::html::{Html, HtmlQuiz};

impl HtmlQuiz for WereTheyAliveWhenGame {
    fn generate_html_quiz(&mut self, question_count: usize) -> Result<(Html, Html), ()> {
        // Get Qs
        let mut questions = Vec::new();
        let mut rng = rand::thread_rng();
        loop {
            // TODO: bounds checking (is there a .get() or similar?)
            let person = self.people_pool.partial_shuffle(&mut rng, 1).0[0].clone();
            let not_person = self.not_people_pool.partial_shuffle(&mut rng, 1).0[0].clone();
            if let Ok(question) = generate_text_question(person, not_person) {
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
            vec!["", "Question", "", ""],
        ));
        html_answers.push(Html::html_opening_quiz_doc(
            "Quiz Answers",
            vec!["", "Question", "Answer"],
        ));

        // Create HTML tables for Qs and As
        for (i, question) in questions.iter().enumerate() {
            html_quiz.push(Html::quiz_table_row(vec![
                &i.to_string(),
                &question.text,
                "T",
                "F",
            ]));

            html_answers.push(Html::quiz_table_row(vec![
                &i.to_string(),
                &question.text,
                &question.answer.to_string(),
            ]));
        }

        // Finish HTML docs
        html_quiz.push(Html::quiz_html_doc_finish());
        html_answers.push(Html::quiz_html_doc_finish());

        // Return the HTML
        Ok((Html::from_vec(html_quiz), Html::from_vec(html_answers)))
    }
}
