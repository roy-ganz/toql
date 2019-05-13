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

 macro_rules! bad_request_template {
    ($description:expr) => (
        format!(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>400 Bad Request</title>
            </head>
            <body align="center">
                <div align="center">
                    <h1>400: Bad Request</h1>
                    <p>Request failed, because {}.</p>
                    <hr />
                    <small>Rocket</small>
                </div>
            </body>
            </html>
        "#, $description
        )
    )
}

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
                response.set_sized_body(Cursor::new(bad_request_template!(err)));
                Ok(response)
            }
              ToqlError::EncodingError(err) => {
                log::info!("{}", err);
               response.set_status(Status::BadRequest);
                response.set_sized_body(Cursor::new(bad_request_template!(err)));
                Ok(response)
             }
             ToqlError::QueryParserError(err) => {
                log::info!("{}", err);
                response.set_status(Status::BadRequest);
                response.set_sized_body(Cursor::new(bad_request_template!(err)));
                Ok(response)
             }
            ToqlError::NotUnique => {
                log::info!("No unique result found");
                response.set_status(Status::BadRequest);
                response.set_sized_body(Cursor::new(bad_request_template!("no unique record found")));
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
       
       println!("VALUE IS `{}`", form_value.url_decode().unwrap());
       if form_value.len() == 0 {
            return Ok(Query::wildcard());  
       }
       let query = form_value.url_decode();
       match  query {
           Err(err) => Err(ToqlError::EncodingError(err)),
           Ok(q) =>  QueryParser::parse(&q)
       }
      
    }
}