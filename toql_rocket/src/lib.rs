



use rocket::FromForm;

#[derive(FromForm, Debug)]
pub struct ToqlQuery {
    query: Option<String>,
    first: Option<u64>,
    max: Option<u16>,
    count: Option<bool>,
    distinct: Option<bool>
}

#[cfg(feature = "mysqldb")]
pub mod mysql {
    use toql_core::load::LoadError;
    use toql_core::sql_mapper::SqlMapperCache;
    use toql_core::query_parser::QueryParser;
    use toql_core::query::Query;

    use mysql::Conn;
    use toql_mysql::load::Load;
    use rocket::http::Status;

    
    use rocket::response::Response;
    use std::io::{Read, Seek};
    use super::ToqlQuery;


    pub fn load_one<'a,T: Load<T>, B>(mut query: &mut Query, mappers: &SqlMapperCache, mut conn: &mut mysql::PooledConn, serialize: &Fn(&T) -> B, distinct: bool) 
    -> Response<'a> 
    where  B: Read + Seek + 'a
    {

        let result = T::load_one(&mut query, &mappers, &mut conn, distinct);
        let mut response = rocket::response::Response::new();
        match result {
            Ok(entity) => {
                    
                    response.set_sized_body(serialize(&entity));
                    response
                },
            Err(LoadError::NotFound) => {
                log::warn!("No result found for Toql query \"{}\"", query.to_string());
                response.set_status(Status::NotFound);
                response
            },
            Err(LoadError::NotUnique) =>{
                log::error!("No unique result found for Toql query \"{}\"",query.to_string() );
                response.set_status(Status::BadRequest);
                response
            }
        }   

    }

    pub fn load_many<'a,T: Load<T>, B>(toql_query: &ToqlQuery, mappers: &SqlMapperCache, mut conn: &mut mysql::PooledConn, serialize: &Fn(&[T]) -> B) 
    -> Response<'a>  
    where  B: Read + Seek + 'a
    {
        
        let query_string = toql_query.query.as_ref().map(|x| &**x).unwrap_or("*");
    
        let mut query = QueryParser::parse(query_string).unwrap();

            // Returns sql errors
        let result = T::load_many(&mut query, &mappers, &mut conn, 
            toql_query.distinct.unwrap_or(false), 
            toql_query.count.unwrap_or(true) ,  
            toql_query.first.unwrap_or(0), 
            toql_query.max.unwrap_or(10));
    
        
        let mut response = Response::new();
        match result {
            Ok((entities, count)) => {
                
                    if let Some((total_count, filtered_count)) = count {
                        response.adjoin_raw_header("X-Count", filtered_count.to_string());
                        response.adjoin_raw_header("X-Total-Count", total_count.to_string());
                    }
                    response.set_sized_body(serialize(&entities));
                    response 
                },
            Err(x) =>{
                log::error!("Invalid SQL = {}",x );
                
                response.set_status(Status::InternalServerError);
                response
                
                
                //InternalError(format!("No results for query \"{}\"", query_string))
            }
        }
    }
}

