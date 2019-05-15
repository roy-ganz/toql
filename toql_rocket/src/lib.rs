use rocket::FromForm;
use rocket::Request;

use rocket::response::Responder;
use rocket::Response;
use rocket::http::Status;
use toql_core::query::Query;

#[derive(FromForm, Debug)]
pub struct ToqlQuery {
    pub query: Option<Query>,
    pub first: Option<u64>,
    pub max: Option<u16>,
    pub count: Option<bool>,
}


#[derive( Debug)]
pub struct Counted<R>(pub R, pub Option<(u32, u32)>);

impl<'r, R: Responder<'r>> Responder<'r> for Counted<R> 
{
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        let mut build = Response::build();
        let responder = self.0;
        build.merge(responder.respond_to(req)?);

        if let Some((total_count, filtered_count)) = self.1 {
            build.raw_header("X-Total-Count", total_count.to_string());
            build.raw_header("X-Filtered-Count", filtered_count.to_string());
        }
           
         build.ok()
    }
}



#[cfg(feature = "mysqldb")]
pub mod mysql {
    use super::ToqlQuery;
    use super::Query;
    use toql_core::error::ToqlError;
    use toql_core::sql_mapper::SqlMapperCache;
    use toql_mysql::load::Load;
    
    pub fn load_many<'a, T: Load<T>>(
        toql_query: &ToqlQuery,
        mappers: &SqlMapperCache,
        mut conn: &mut mysql::Conn
    ) 
    -> Result<(Vec<T>, Option<(u32, u32)>), ToqlError>
    {
        // Returns sql errors
        T::load_many(
            &toql_query.query.as_ref().unwrap_or(&Query::wildcard()),
            &mappers,
            &mut conn,
            toql_query.count.unwrap_or(true),
            toql_query.first.unwrap_or(0),
            toql_query.max.unwrap_or(10),
        )
    }
    
}
