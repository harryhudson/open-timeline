// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! HTML quiz creation
//!

/// Implementing types can produce an HTML quiz (with answers)
pub trait HtmlQuiz {
    // TODO: can this be derived for all games by implementing a ToHtml trait?
    /// Create a HTML quiz (and answers)
    fn generate_html_quiz(&mut self, question_count: usize) -> Result<(Html, Html), ()>;
}

/// For HTML creation
pub struct Html(pub String);

// TODO: check column counts?
impl Html {
    /// Get the underlying `&str`
    pub fn str(&self) -> &str {
        &self.0
    }

    /// Get a single HTML string from a vector of HTML strings
    pub fn from_vec(html: Vec<Html>) -> Self {
        Html(
            html.into_iter()
                .map(|html| html.0)
                .collect::<Vec<String>>()
                .concat(),
        )
    }

    /// Begin HTML docs
    pub fn html_opening_quiz_doc(
        title: impl ToString,
        table_column_headings: Vec<impl ToString>,
    ) -> Self {
        let title = title.to_string();
        let table_column_headings: Vec<String> = table_column_headings
            .into_iter()
            .map(|heading| heading.to_string())
            .collect();
        let mut html = format!(
            r"
                <h1>{title}</h1>
                <table>
                    <tr>
                        <th></th>
                        <th>Question</th>
                        <th></th>
                        <th></th>
                        <th></th>
                    </tr>
            "
        );
        for heading in table_column_headings {
            html.push_str(&format!("<th>{heading}</th>"));
        }
        html.push_str("</tr>");
        Html(html)
    }

    pub fn quiz_table_row(table_column_content: Vec<impl ToString>) -> Self {
        let mut row = String::from("<tr>");
        for column in table_column_content {
            row.push_str(&format!("<td>{}</td>", column.to_string()));
        }
        row.push_str("</tr>");
        Html(row)
    }

    pub fn quiz_html_doc_finish() -> Self {
        Html(String::from("</table>"))
    }
}
