use toql_core::query::Query;
use rocket::FromForm;

#[derive(FromForm, Debug)]
pub struct ToqlQuery {
    pub query: Option<Query>,
    pub first: Option<u64>,
    pub max: Option<u16>,
    pub count: Option<bool>,
}

