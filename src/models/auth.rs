use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct SigninReq {
    #[validate(email(message = "invalid email format"))]
    pub email: String,

    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}
