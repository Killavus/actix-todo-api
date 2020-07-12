use actix_web::{error, http::StatusCode, HttpResponse, ResponseError};
use std::error::Error;
use std::fmt::{self, Display};
use validator::ValidationErrors;

use serde_json::{json, Value};

#[derive(Debug)]
pub enum WebError {
  ValidationError(ValidationErrors),
  DatabaseError(sqlx::Error),
  ActixError(error::Error),
}

impl Error for WebError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    use WebError::*;

    match self {
      ActixError(err) => Some(err),
      _ => None,
    }
  }
}

impl Display for WebError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use WebError::*;
    let identifier = match self {
      ValidationError(_) => "ValidationError",
      DatabaseError(_) => "DatabaseError",
      ActixError(_) => "InternalError",
    };

    write!(f, "{}", identifier)
  }
}

impl WebError {
  fn populate_error_map(&self, error_map: &mut Value) {
    use WebError::*;
    error_map["type"] = json!(self.to_string());

    match self {
      ValidationError(errors) => {
        error_map["details"] =
          serde_json::to_value(errors).unwrap_or(json!({ "error": "unknown validation error" }));
      }
      DatabaseError(err) => {
        error_map["details"] = json!({ "message": err.to_string() });
      }
      ActixError(err) => {
        error_map["details"] = json!({ "message": err.to_string() });
      }
    }
  }
}

impl ResponseError for WebError {
  fn status_code(&self) -> StatusCode {
    use WebError::*;

    match self {
      ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
      _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }

  fn error_response(&self) -> HttpResponse {
    let mut error_map = json!({});
    self.populate_error_map(&mut error_map);

    HttpResponse::build(self.status_code())
      .content_type("application/json")
      .json(error_map)
  }
}
