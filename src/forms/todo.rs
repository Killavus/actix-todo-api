use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTodo {
  #[validate(length(min = 1))]
  pub content: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTodo {
  #[validate(length(min = 1))]
  pub content: Option<String>,
  pub completed: Option<bool>,
}
