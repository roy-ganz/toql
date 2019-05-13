#Rocket integration

To enable rocket integration you must use the feature

[dependencies]
toql = {version="0.1, features=["rocket_mysql"]


## Functionality

The toql rocket library provides the type ToqlQuery and a load function to run the query.
Here is a small example.

#[derive(Toql)]
    struct User {
      id: u32,
      name: Option<String>
    }
    
    #[query("/?<toql..>")]
	fn query(toql: Form<ToqlQuery>, mappers: State<SqlMapperCache>) -> Result<Counted<Json<User>>> {
       
        ler r = toql::rocket::load_many(toql, mappers)?;
        Ok(Counted(Json(r.0), r.1))
	}

	fn main() {
	    let mapper = toql::sql_mapper::SQLMapper::map<User>();
    	rocket::ignite().mount("/query", routes![query]).launch();
	}

## URL query parameters
ToqlQuery has the following parameters
query   the query string
offset
max
counts  to run count statistics
  
Example:

localhost:8000/user/query?toql=name LK "FOO%";name LK "BAR%"&max=5&offset=10&count=1

## Load function
To load 


## Error handling
The rocket integration supports toql::error::Result and will convert errors into approproate 
