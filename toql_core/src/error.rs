
use std::fmt;
use crate::sql_builder::SqlBuilderError;

 #[cfg(feature = "mysqldb")]
use mysql::error::Error;

#[derive(Debug)]
 pub enum ToqlError {
    NotFound, 
    NotUnique,
    MapperMissing(String),
    SqlBuilderError(SqlBuilderError),

    #[cfg(feature = "mysqldb")]
    MySqlError(Error)
} 

 #[cfg(feature = "rocketweb")]
impl rocket::response::Responder<'static> for ToqlError {
    fn respond_to(self, _: &rocket::Request) -> Result<rocket::Response<'static>, rocket::http::Status> {
        let mut response = rocket::response::Response::new();
        match self {
            ToqlError::NotFound => {
                // TODO add query to not found
                log::info!("No result found for Toql query `{}`", "TDDO");
                response.set_status(rocket::http::Status::NotFound);
                Ok(response)
            }
            ToqlError::NotUnique => {
                 // TODO add query to not found
                log::info!( "No unique result found for Toql query `{}`", "TODO");
                response.set_status(rocket::http::Status::BadRequest);
                Ok(response)
            },
            err => {
                log::error!("Toql failed with `{}`",err);
                Err(rocket::http::Status::InternalServerError)
            }
        }

    }
}

impl From<SqlBuilderError> for ToqlError {
        fn from(err: SqlBuilderError) -> ToqlError {
        ToqlError::SqlBuilderError(err)
    }
}

#[cfg(feature = "mysqldb")]
impl From<Error> for ToqlError {
        fn from(err: Error) -> ToqlError {
        ToqlError::MySqlError(err)
    }
}

impl fmt::Display for ToqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ToqlError::NotFound =>
                write!(f, "no result found"),
            ToqlError::NotUnique =>
                write!(f, "no unique result found"),
            ToqlError::MapperMissing(ref s) =>
                write!(f, "no mapper found for `{}`", s),
            #[cfg(feature = "mysqldb")]
            ToqlError::MySqlError (ref e) => e.fmt(f),
            ToqlError::SqlBuilderError (ref e) => e.fmt(f),
        }
    }
}