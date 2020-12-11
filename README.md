# Toql

### Description
Toql *Transfer object query language* is a query language to build SQL statements. It can retrieve filtered, ordered and indiviually selected columns from a database and put the result into your structs.

Toql turns this
```toql
id, (+age eq 16; age eq 18), address_street
```
into
```sql
SELECT user.id, user.age, address.street
FROM User user LEFt JOIN Address address ON (user.address_id= address.id)
WHERE user.age = 16 OR user.age = 18
ORDER BY user.age ASC
```
for all your `Toql` derived structs.

### Resources
There is also a [guide](https://roy-ganz.github.io/toql) and the [API documentation](https://docs.rs/toql/0.3/toql/).

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
toql = { version = "0.3" }
toql_mysql = { version = "0.1" }
```

### Integration 
[Toql Rocket](https://crates.io/crates/toql_rocket) plays well with [Rocket](https://crates.io/crates/rocket): Add this to your `Cargo.toml`

```toml
[dependencies]
toql_rocket = { version = "0.1", features = ["mysql"] }
```

Right now there is only support for `MySQL`. Add `features = ["mysql"]` to your `Cargo.toml` dependencies.

## Features
 - Can query, insert, update and delete single and multiple database records.
 - Handles dependencies in queries through SQL joins and merges. Cool!
 - Is fast, beause the mapper is only created once and than reused.
 - Has high level functions for speed and low level functions for edge cases.
 - Easy to use.
 

## Contribution
My near term goal is to support for more web frameworks and databases. However I would like to stabilize the API first. So you are welcome to play around and test it (**don't use it in production yet**). Comments, bug fixes and quality improvements are welcome. For features please hold on.

## Other database projects
- [Diesel](http://diesel.rs/)
- [GraphQL](https://github.com/graphql-rust)


## Background
I have developed the initial Toql language about 7 years ago for a web project. I have refined it since then and you can see it in action on [www.schoolsheet.com](https://www.schoolsheet.com), a web service I created in Java for my teaching. The Rust implementation is must faster though ;)


## License

Toql is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

