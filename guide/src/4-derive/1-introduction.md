# The Toql Derive
The recommended way to map Toql fields onto Rust structs is to use the Toql derive.

The derive builds a lot of code. This includes

- Mapping of Toql fields to struct fields and database columns or expressions.
- Creating field methods for the query builder.
- Handling relationships through joins and merges.
- Creating high level functions to load, insert, update and delete structs.


## Example

With this simple code

 ```rust
	#[derive(Toql)]
	struct User {
		#[toql(key)]
		id: u32,
		name: Option<String>
}
```

We can now do the following

```rust
use toql::mysql::load_one; // Load function from derive
use toql::mysql::update_one; // Update function from derive

let conn = --snip--
let cache = TableMapperRegistry::new();
TableMapper::insert_new_mapper::<User>(&mut cache); // Mapper function from derive

let q = Query::wildcard().and(User::fields.id().eq(5)); // Builder fields from derive
let user = load_one<User>(&q, &cache, &mut conn); 

user.age = Some(16);
update_one(&user); 
```
