use rocket::FromForm;
use rocket::Request;

use rocket::response::Responder;
use rocket::Response;
use rocket::http::Status;
use toql_core::query::Query;

#[derive(FromForm, Debug)]
pub struct ToqlQuery {
    pub query: Query,
    pub first: Option<u64>,
    pub max: Option<u16>,
    pub count: Option<bool>,
    //pub distinct: Option<bool>,
    
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

//impl rocket::response::Responder<'r> for  Counted {}




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


  


   /*  pub fn load_response<'a, T, B>( result: (Vec<T>, Option<(u32, u32)>) + 'a, response: &mut Response,serialize: &Fn(&[T]) -> B)
    where B: Read + Seek + 'a,
     {
         if let( Some(count)) = result.1 {
                if let (total_count, filtered_count) = count {
                    response.adjoin_raw_header("X-Count", filtered_count.to_string());
                    response.adjoin_raw_header("X-Total-Count", total_count.to_string());
                }
                response.set_sized_body(serialize(&result.0));
        }
         response.set_sized_body(serialize(&result.0));
    } */

    pub fn load_many<'a, T: Load<T>>(
        toql_query: &ToqlQuery,
        mappers: &SqlMapperCache,
        mut conn: &mut mysql::Conn
        //serialize: &Fn(&[T]) -> B,
    ) 
    //-> Result<Response<'a>, ToqlError>
    -> Result<(Vec<T>, Option<(u32, u32)>), ToqlError>
    /* where
        B: Read + Seek + 'a, */
    {
       
        // Returns sql errors
        T::load_many(
            &toql_query.query,
            &mappers,
            &mut conn,
            toql_query.count.unwrap_or(true),
            toql_query.first.unwrap_or(0),
            toql_query.max.unwrap_or(10),
        )
       /*  // Returns sql errors
        let result = T::load_many(
            &toql_query.query,
            &mappers,
            &mut conn,
            toql_query.count.unwrap_or(true),
            toql_query.first.unwrap_or(0),
            toql_query.max.unwrap_or(10),
        )?; */

       /*  let mut response = Response::new();
        if let( Some(count)) = result.1 {
                if let (total_count, filtered_count) = count {
                    response.adjoin_raw_header("X-Count", filtered_count.to_string());
                    response.adjoin_raw_header("X-Total-Count", total_count.to_string());
                }
                response.set_sized_body(serialize(&result.0));
        }
         response.set_sized_body(serialize(&result.0));
         Ok(response) */
        /* match result.1 {
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
        } */
    }
    
}
