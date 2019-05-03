use crate::query::Query;
use crate::query_parser::QueryParser;
use rocket::request::FromFormValue;
use rocket::http::RawStr;
use crate::error::ToqlError;
use rocket::http::Status;
use rocket::Response;
use rocket::Request;
use crate::pest::error::Error;

impl rocket::response::Responder<'static> for ToqlError {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        let mut response = Response::new();
        match self {
            ToqlError::NotFound => {
                // TODO add query to not found
                log::info!("No result found");
                response.set_status(Status::NotFound);
                Ok(response)
            }
            ToqlError::NotUnique => {
                 // TODO add query to not found
                log::info!( "No unique result found");
                response.set_status(Status::BadRequest);
                Ok(response)
            },
            err => {
                log::error!("Toql failed with `{}`",err);
                Err(rocket::http::Status::InternalServerError)
            }
        }

    }
}


impl<'v> FromFormValue<'v> for Query {
    type Error = ToqlError;

    fn from_form_value(form_value: &'v RawStr) -> Result<Query, ToqlError> {
       
       if form_value.len() == 0 {
            return QueryParser::parse("*")    
       }
       QueryParser::parse(&form_value)

       /*  match form_value.parse::<String>() {
            Ok(query) =>  QueryParser::parse(&query),
            _ => ToqlErrror::Other(form_value),
        } */
    }
}