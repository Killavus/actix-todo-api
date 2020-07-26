use serde::Deserialize;
use validator::Validate;
#[derive(Debug, Validate, Deserialize)]
pub struct CreateWorkList {
    #[validate(length(min = 1))]
    pub name: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateWorkList {
    #[validate(length(min = 1))]
    pub name: String,
}
