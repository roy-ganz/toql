# Toql

### Description
Toql *Transfer object query language* is a query language to build SQL statements to retrieve filtered, ordered and indiviually selected columns from a database. It's aim is to give web clients an easy way to get data from a server over a REST interface.

Toql is **not** an ORM. It's purpose is not to hide SQL but to make it more easy to build boilerplate SQL. *Don't be afraid of SQuirreLs 🐿️*

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
toql ="0.1"
```

## Project

The toql project consists of 

* A __query parser__, typically used to parse query string from web clients.
* A __query builder__, used to modify or create queries on the fly.
* An __SQL mapper__, that translates toql fields into SQL columns or expressions.
* A __SQL builder__, that turns a query into SQL with the hep of the mapper.
* A __toql derive__ that generates mappings of structs, functions to handle dependencies and helper functions.
* __3rd party integration __  to work well together with Rocket and MySQL.

Make sure you check out the [guide]() or run the example of a full CRUD with Rocket and MySQL. 

```bash
cargo run --example="rocket_mysql"
```


## Contribution
My near term goal is to support for more web frameworks and databases. However I would like to stabilise the API first. So you are welcome to play around and test it (**don't use it in production yet**). Comments, bug fixes and quality improvements are welcome. For features please hold on.

## Related projects
[Diesel](www.http://diesel.rs/) is an  ORM for RUST
[GraphQL](https://github.com/graphql-rust) can query data.

## Background

I developped the initial Toql language about 5 years ago for a web project. I have refined it since then and you can see it in action on [www.schoolsheet.com] (www.schoolsheet.com), a webservice I created in JAVA for my teaching. The Rust implementation is must faster though ;)


## License

mysql_enum is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

