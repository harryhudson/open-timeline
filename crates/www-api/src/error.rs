// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! API error response
//!

use crate::helpers::ErrorMsg;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use open_timeline_crud::CrudError;

/// Container for API errors.  Can be sent back to the client
pub struct ApiError(pub (StatusCode, Json<ErrorMsg>));

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(value: sqlx::Error) -> Self {
        let value: CrudError = value.into();
        value.into()
    }
}

impl From<CrudError> for ApiError {
    fn from(value: CrudError) -> Self {
        ApiError((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorMsg {
                error_msg: value.to_string(),
            }),
        ))
    }
}
