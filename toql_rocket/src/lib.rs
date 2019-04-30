use rocket::FromForm;

#[derive(FromForm, Debug)]
pub struct ToqlQuery {
    query: Option<String>,
    first: Option<u64>,
    max: Option<u16>,
    count: Option<bool>,
    distinct: Option<bool>,
}

#[cfg(feature = "mysqldb")]
pub mod mysql {
    use toql_core::error::ToqlError;
    use toql_core::query::Query;
    use toql_core::query_parser::QueryParser;
    use toql_core::sql_mapper::SqlMapperCache;

    use rocket::http::Status;
    use toql_mysql::load::Load;
    use toql_mysql::alter::Alter;

    use super::ToqlQuery;
    use rocket::response::Response;
    use std::io::{Read, Seek};

    pub fn load_one<'a, T: Load<T>, B>(
        mut query: &mut Query,
        mappers: &SqlMapperCache,
        mut conn: &mut mysql::Conn,
        serialize: &Fn(&T) -> B,
        distinct: bool,
    ) -> Response<'a>
    where
        B: Read + Seek + 'a,
    {
        let result = T::load_one(&mut query, &mappers, &mut conn, distinct);
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
        let query_string = toql_query.query.as_ref().map(|x| &**x).unwrap_or("*");

        let mut query = QueryParser::parse(query_string).unwrap();

        // Returns sql errors
        let result = T::load_many(
            &mut query,
            &mappers,
            &mut conn,
            toql_query.distinct.unwrap_or(false),
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
    pub fn insert_one<'a, T:'a + Alter<'a,T>> (entity: T, mut conn: &mut mysql::Conn) -> Response<'a> {

         let mut response = Response::new();
         let result = toql_mysql::insert_one(&entity, conn);
         match result {
            Ok(_last_insert_id) => {
                response.set_status(Status::Created);
                response
            }
            Err(x) => {
                    log::error!("Toql failed with `{}`", x);
                    response.set_status(Status::InternalServerError);
                    response
            }
         }
    }
    
    pub fn update_one<'a, T:'a + Alter<'a,T>> (entity: T, mut conn: &mut mysql::Conn) -> Response<'a> {
        let mut response = Response::new();
         let result = toql_mysql::update_one(&entity, conn);
         match result {
            Ok(_rows_updated) => {
                response.set_status(Status::Ok);
                response
            }
            Err(x) => {
                    log::error!("Toql failed with `{}`", x);
                    response.set_status(Status::InternalServerError);
                    response
            }
        }
    }
    
    pub fn delete_one<'a, T:'a + Alter<'a,T>> (entity: T, mut conn: &mut mysql::Conn) -> Response<'a> {
        let mut response = Response::new();
         let result = toql_mysql::delete_one(&entity,conn);
         match result {
            Ok(_rows_deleted) => {
                response.set_status(Status::Ok);
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
