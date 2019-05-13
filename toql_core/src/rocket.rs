use crate::query::Query;
use crate::query_parser::QueryParser;
use rocket::request::FromFormValue;
use rocket::http::RawStr;
use crate::error::ToqlError;
use rocket::http::Status;
use rocket::Response;
use rocket::Request;
use crate::pest::error::Error;
use std::io::Cursor;

impl rocket::response::Responder<'static> for ToqlError {

    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        let mut response = Response::new();
        match self {
            ToqlError::NotFound => {
                log::info!("No result found");
                Err(Status::NotFound)
            }
            ToqlError::SqlBuilderError(err) => {
                log::info!("{}", err);
                response.set_status(Status::BadRequest);
                response.set_sized_body(Cursor::new(format!("
                    <!DOCTYPE html>\
                    <html>\n\
                    <head>\n\
                        <meta charset=\"utf-8\">\n\
                        <title>400 Bad Request</title>\n\
                    </head>\n\
                    <body align=\"center\">\n\
                        <div align=\"center\">\n\
                            <h1>400: Bad Request</h1>\n\
                            <p>Request failed becuase {}.</p>\n\
                            <hr />\n\
                            <small>Rocket</small>\n\
                        </div>\
                    </body>\
                    </html>", err)
                ));
                Ok(response)
                //Err(Status::BadRequest)
            }
             ToqlError::QueryParserError(err) => {
                log::info!("{}", err);
                Err(Status::BadRequest)
             }
            ToqlError::NotUnique => {
                log::info!("No unique result found");
                Err(Status::BadRequest)
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
            return Ok(Query::wildcard());  
       }
       QueryParser::parse(&form_value)
    }
}