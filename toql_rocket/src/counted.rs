use rocket::http::Status;
use rocket::Response;
use rocket::Request;
use rocket::response::Responder;

/// Struct that keeps a result with counting information.
#[derive( Debug)]
pub struct Counted<R>(pub R, pub Option<(u32, u32)>);

/// Creates a response and puts the counting information
/// into the HTTP response headers, if there is some value.
/// 
/// The header `X-Total-Count` contains the first value of the tuple in [Counted].
/// The header `X-Filtered-Count` contains the second value of the tuple in [Counted].
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