
Toql - A friendly and productive ORM
==========================================================


[Beginner Guide](https://roy-ganz.github.io/toql), [API documentation](https://docs.rs/toql/0.3/toql/)

Toql (pronounced *to-cue-ell*) is an ORM for async databases that features
- Translation between Rust structs and database tables.
- Can load and modify nested structs.
- A unique dead simple query language, suitable for web clients.
- Different table aliases from long and readable to tiny and fast.
- Prepared statements against SQL Injection
- Support for raw SQL for full database power
- Support for role based access
- Highly customizable through user defined parameters, query functions, field handlers, etc. 
- Compile time safety for queries, fields and path names.
- No unsafe Rust code
- Tested on real world scenario

It currently only supports **MySQL**. More are coming, promised :)

### Installation
Add this to your `Cargo.toml`:

```toml
[dependencies]
toql = { version = "0.3" }
toql_mysql_async = { version = "0.3" }
```

### Integration 
[Toql Rocket](https://crates.io/crates/toql_rocket) plays well with [Rocket](https://crates.io/crates/rocket): Add this to your `Cargo.toml`

```toml
[dependencies]
toql_rocket = { version = "0.1", features = ["mysql"] }
```
Right now there is only support for `MySQL`. Add `features = ["mysql"]` to your `Cargo.toml` dependencies.

## Look and feel
```rust
#[derive(Toql)]
#[toql(auto_key = true)]
struct Todo {
    #[toql(key)]
    id: u64,
    what: String,

    #[toql(join())]
    user: User
}

// --snip--
let toql = ...
let todo = ...

// Insert todo and update its generated id
toql.insert_one(&mut todo, paths!(top)).await?; 

// Compile time checked queries!
let q = query!(Todo, "*, user_id eq ?", &todo.user.id); 

// Typesafe loading
let user = toql.load_many(q).await?; 
// --snip--
```

## Contribution
My near term goal is to support for more web frameworks and databases. However I would like to stabilize the API first. So you are welcome to play around and test it (**don't use it in production yet**). Comments, bug fixes and quality improvements are welcome. For features please hold on.


## License
Toql is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

