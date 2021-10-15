# Concept

Toql is a ORM that aim to boost your developer comfort and speed when working with databases.

To use it you must derive `Toql` for all structs that represent a table in your database:
- The fields of those structs represent either columns, SQL expressions or 
relationships to other tables.
- The fields also determine the field name or in case of a relationship the path name in the [Toql query](5-query-language/1-introduction.md)

A struct may map only some columns of a table and also multiple structs may refer to the same table. Structs are merly 'views' to a table.

A derived struct can then be inserted, updated, deleted and loaded from your database. To do that you must call the [Toql API functions](3-api/1-introduction.md) with a query string or just a list of fields or paths.

Here the typical flow in a web environment:
1. A web client sends a Toql query to the REST Server.
2. The server uses Toql to parse the query and create SQL statements.
3. Toql sends the SQL to the database
4. and deserializes the resulting rows into Rust structs.
4. The server sends these structs to the client.

## Example

below a full example that uses Rocket to serve users from a database. 
Notice the two Toql derived structs at the beginning. The rest of the code is fairly boilerplate.

```rust
	// Toql derived structs
	#[derive(Toql)]
	struct Country {
		#[toql(key)]
		id: String,
		name: Option<String>
	}

	#[derive(Toql)]
	#[toql(auto_keys = true)]
	struct User {
		#[toql(key)]
		id: u32,
		name: Option<String>,
		#[toql(join())]
		country: Option<Country>
	}
    
	// Here Rocket with some Toql integration (ToqlQuery and Counted)
	#[query("/?<query..>")]
	fn query(query: Form<ToqlQuery>, mut conn: Connection<ExampleDb>, 
		cache: State<Cache> {
		
		let toql = MySqlAsync::from(&mut *conn, &cache);

		let r = toql.load_page(query, page)?;
		Ok(Counted(Json(r.0), r.1))
	}

	#[database("example_db")]
	pub struct ExampleDb(mysql::Conn);

	fn main() {
		rocket::ignite().mount("/query", routes![query]).launch();
	}
```

with `Cargo.toml`
```
toql = v0.3
toql_rocket = "0.3"
toql_mysql_async = "0.3"
rocket = "0.5"
mysql_async = "0.20"
```

If you have a MySQL Server running, try the full CRUD example.

```bash
ROCKET_DATABASES={example_db={url=mysql://USER:PASSWORD@localhost:3306/example_db}} cargo run --example crud_rocket_mysql

```


