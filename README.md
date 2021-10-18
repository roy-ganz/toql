
![Bollard by the Sea by Gábor Szakács (Public Domain)](book/src/bollard-at-the-port.gif)

# Toql - A friendly and productive ORM


[Beginner Guide](https://roy-ganz.github.io/toql), [API documentation](https://docs.rs/toql/0.3/toql/)

Toql is an ORM for async databases that features
- Translation between Rust structs and database tables.
- Can load and modify nested structs.
- A unique dead simple query language, suitable for web clients.
- Different table aliases from long and readable to tiny and fast.
- Prepared statements against SQL injection.
- Support for raw SQL for full database power.
- Support for role based access.
- Highly customizable through user defined parameters, query functions, field handlers, etc. 
- Compile time safety for queries, fields and path names.
- No unsafe Rust code.
- Tested on real world scenario.

It currently only supports **MySQL**. More are coming, promised :)

## Installation
Add this to your `Cargo.toml`:

```toml
[dependencies]
toql = {version = "0.3", features = ["serde"]}
toql_mysql_async = "0.3"
mysql_async = "0.27"
```

## Look And Feel

Derive your structs:
```rust
#[derive(Toql)]
#[toql(auto_key = true)]
struct Todo {
    #[toql(key)]
    id: u64,
    what: String,

    #[toql(join())]
    user: Join<User> // Join type isn't required, but may improve experience
}
```

And do stuff with them:
```rust
let toql = ...
let todo = Todo{ id:0, 
                 what: "Water plants".to_string(), 
                 user: Join::with_key(5.into())};

// Insert todo and update its generated id
toql.insert_one(&mut todo, paths!(top)).await?; 

// Compile time checked queries!
let q = query!(Todo, "*, user_id eq ?", &todo.user.id); 

// Typesafe loading
let user = toql.load_many(q).await?; 
```


## Quick start
Toql has a [supporting crate](https://crates.io/crates/toql_rocket) to play well with [Rocket](https://crates.io/crates/rocket). Check out the [CRUD example](https://github.com/roy-ganz/todo_rotomy).

## Contribution
Comments, bug fixes and quality improvements are welcome. 

## License
Toql is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

