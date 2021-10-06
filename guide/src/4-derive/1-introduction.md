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

let toql = --snip--
let cache = 

let q = query!(User, "id eq 5"); 
let mut user = toql.load_one(&q); 

user.age = Some(16);
toql.update_one(&mut user); 
```
