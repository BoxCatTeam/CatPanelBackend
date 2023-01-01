use crate::http::error::AnyResult;

pub async fn hello_world() -> AnyResult<&'static str> {
    Ok("Hello, World!")
}
