use crate::http::model::system_info::SystemInfo;
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

mod system_info;

pub struct Query;

#[Object]
impl Query {
    async fn system_info(&self) -> SystemInfo {
        SystemInfo::default()
    }
}

pub fn schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}

#[test]
fn print_sdl() {
    println!("{}", schema().sdl());
}

#[cfg(test)]
mod tests {
    use crate::http::model::schema;

    #[test]
    fn test_system_info() {
        todo!()
    }
}
