use rocket::FromForm;
use toql_core::query::Query;

#[derive(FromForm, Debug)]
pub struct ToqlQuery {
    pub query: Query,
    pub first: Option<u64>,
    pub max: Option<u16>,
    pub count: Option<bool>,
    //pub distinct: Option<bool>,
    
}

#[cfg(feature = "mysqldb")]
pub mod mysql {
    use toql_core::error::ToqlError;
    use toql_core::query::Query;
    use toql_core::query_parser::QueryParser;
    use toql_core::sql_mapper::SqlMapperCache;

    use rocket::http::Status;
     use rocket::Request;
     use rocket::response::Responder;


    use toql_mysql::load::Load;
    
    use super::ToqlQuery;
    use rocket::response::Response;
    use std::io::{Read, Seek};




    pub fn load_one<'a, T: Load<T>, B>(
        mut query: &mut Query,
        mappers: &SqlMapperCache,
        mut conn: &mut mysql::Conn,
        serialize: &Fn(&T) -> B
    ) -> Response<'a>
    where
        B: Read + Seek + 'a,
    {
        let result = T::load_one(&mut query, &mappers, &mut conn);
        let mut response = rocket::response::Response::new();
        match result {
            Ok(entity) => {
                response.set_sized_body(serialize(&entity));
                response
            }
            Err(ToqlError::NotFound) => {
                log::info!("No result found for Toql query `{}`", query);
                response.set_status(Status::NotFound);
                response
            }
            Err(ToqlError::NotUnique) => {
                log::info!( "No unique result found for Toql query `{}`",query);
                response.set_status(Status::BadRequest);
                response
            },
            Err(err) => {
                 log::error!("Toql failed with `{}`",err);
                response.set_status(Status::InternalServerError);
                response
            }
        }
    }

    pub fn load_many<'a, T: Load<T>, B>(
        toql_query: &ToqlQuery,
        mappers: &SqlMapperCache,
        mut conn: &mut mysql::Conn,
        serialize: &Fn(&[T]) -> B,
    ) -> Response<'a>
    where
        B: Read + Seek + 'a,
    {
       // let query_string = toql_query.query.as_ref().map(|x| &**x).unwrap_or("*");

        //let mut query = QueryParser::parse(query_string).unwrap();
        

        // Returns sql errors
        let result = T::load_many(
            &toql_query.query,
            &mappers,
            &mut conn,
            toql_query.count.unwrap_or(true),
            toql_query.first.unwrap_or(0),
            toql_query.max.unwrap_or(10),
        );

        let mut response = Response::new();
        match result {
            Ok((entities, count)) => {
                if let Some((total_count, filtered_count)) = count {
                    response.adjoin_raw_header("X-Count", filtered_count.to_string());
                    response.adjoin_raw_header("X-Total-Count", total_count.to_string());
                }
                response.set_sized_body(serialize(&entities));
                response
            }
            Err(x) => {
                log::error!("Toql failed with `{}`", x);
                response.set_status(Status::InternalServerError);
                response

                
            }
        }
    }
    
}
