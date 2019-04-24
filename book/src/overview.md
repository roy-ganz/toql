# Toql
Toql _Transfer Object Query Language_ is a library that turns a query string into SQL to retrieve data records. It is useful for web clients to get database records from a REST interface. Toql consists of

* A __query parser__, typically used to parse query string from web clients.
* A __query builder__, used to modify or create queries on the fly.
* An __SQL mapper__, that translates toql fields into SQL columns or expressions.
* A __SQL builder__, that turns a query into SQL with the hep of the mapper.
* A __toql derive__ that generates mappings of structs, functions to handle dependencies and helper functions.
* __3rd party integration __  to work well together with Rocket and MySQL.



Here is a full example of Toql, using Rocket to serve users from a database:

	#[derive(Toql)]
    struct User {
      id: u32,
      name: Option<String>
    }
    
    #[query("/?<toql..>")]
	fn query(toql: Form<ToqlQuery>) -> JsonValue {
        let mapper = SQLMapper::map<user>();
        toql::rocket::query_request(toql, mapper)
	}

	fn main() {
    	rocket::ignite().mount("/query", routes![query]).launch();
	}

Visiting localhost:8000/query, for example, will show you a list of users.  
To get only the user ids in ascending order visit localhost:8000/query?query=+id. For a bigger example using complex dependencies see XXX.

## Guide
While Toql is technically documented in the code, there is a guide that covers
* The Query Language
* The Toql Derive
* The Query Builder
* The SQL Mapper
* Rocket Integration
* MySQL Tricks
* Snippets
