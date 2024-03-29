# Toql - A friendly and productive ORM

![Tests](https://github.com/roy-ganz/toql/actions/workflows/tests.yml/badge.svg)
[![Current Crates.io Version](https://img.shields.io/crates/v/toql.svg)](https://crates.io/crates/toql)

[Beginner Guide](https://roy-ganz.github.io/toql_guide) | [API documentation](https://docs.rs/toql)

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
toql = {version = "0.4", features = ["serde"]}
toql_mysql_async = "0.4"
```

## Look And Feel

Derive your structs:
```rust
#[derive(Toql)]
#[toql(auto_key)]
struct Todo {
    #[toql(key)]
    id: u64,
    what: String,

    #[toql(join)]
    user: User 
}
```

And do stuff with them:
```rust
let toql = ...
let todo = Todo{ ... };

// Insert todo and update its generated id
toql.insert_one(&mut todo, paths!(top)).await?; 

// Compile time checked queries!
let q = query!(Todo, "*, user_id eq ?", &todo.user.id); 

// Typesafe loading
let todos = toql.load_many(q).await?; 
```


## Quick start
Check out the [CRUD example](https://github.com/roy-ganz/todo_rotomy).

## Contribution
Comments, bug fixes and quality improvements are welcome. 

## License
Toql is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

