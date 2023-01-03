use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub type AnyResult<T> = Result<T, Error>;

pub struct Error {
    code: StatusCode,
    msg: String,
}

impl Error {
    pub fn status(code: StatusCode, msg: String) -> Self {
        Error { code, msg }
    }
}

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: value.into().to_string(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.code, self.msg).into_response()
    }
}
