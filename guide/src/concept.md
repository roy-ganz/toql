# Concept

Toql is a set of crates that aim to simplify web development:

1. A web client sends a Toql query to the REST Server.
2. The server uses Toql to parse the query and create SQL.
3. SQL is send to the Database.
4. Toql puts the resulting datasets into Rust structs.
4. The structs are sent to the client.

The Toql derive produces various high level functions, so that common operations can be done with a single function call.
For edge cases all the low level functions are available for the programmer, too.

## Example

Here is an excerpt of code that uses Rocket to serve users from a database. 
Notice the two Toql derived structs at the beginning. The rest of the code is fairly boilerplate.

```rust
	#[derive(Toql)]
	#[toql(skip_indelup)] // No insert / delete / update functionality
	struct Country {
		id: String,
		name: Option<String>
	}

	#[derive(Toql)]
	#[toql(skip_indelup)]
	struct User {
		id: u32,
		name: Option<String>,
		#[toql(sql_join(self="country_id", other="id"))]
		country: Option<Country>
	}
    
	#[query("/?<toql..>")]
	fn query(toql: Form<ToqlQuery>,  conn: ExampleDbConnection, 
		mappers: State<TableMapperRegistry>) -> Result<Counted<Json<User>>> {
		let ExampleDbConnection(mut c) = conn;

		let r = toql::rocket::load_many(toql, mappers, &mut c)?;
		Ok(Counted(Json(r.0), r.1))
	}

	#[database("example_db")]
	pub struct ExampleDbConnection(mysql::Conn);

	fn main() {
		let mut mappers = TableMapperRegistry::new();
		TableMapper::insert_new_mapper::<User>(&mut mappers);

		rocket::ignite().mount("/query", routes![query]).launch();
	}
```

If you have a MySQL Server running, try the full CRUD example.

```bash
ROCKET_DATABASES={example_db={url=mysql://USER:PASSWORD@localhost:3306/example_db}} cargo +nightly run --example crud_rocket_mysql

```


