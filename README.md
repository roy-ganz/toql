# Toql

### Description
Toql *Transfer object query language* is a query language to build SQL statements to retrieve filtered, ordered and indiviually selected columns from a database. It's aim is to give web clients an easy way to get data from a server over a REST interface.

Toql is **not** an ORM. It's purpose is not to hide SQL but to make it more easy to build boilerplate SQL. 
*Don't be afraid of SQuirreLs 🐿️*

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
toql ="0.1"
```

## Project

The Toql project consists of 

* A __query parser__ to parse query string from web clients.
* A __query builder__ to modify or create queries on the fly.
* An __SQL mapper__, that translates Toql fields into SQL columns or expressions.
* A __SQL builder__ to turn a query into SQL with the help of the mapper.
* A __Toql derive__ that generates mappings of structs, functions to handle dependencies and helper functions.
* __3rd party integration__  to work well together with Rocket and MySQL.

Make sure you check out the [guide](https://github.com/roy-ganz/toql/blob/master/guide/src/introduction.md) or run the example of a full CRUD with Rocket and MySQL. 

```bash
ROCKET_DATABASES={example_db={url=mysql://USER:PASS@localhost:3306/example_db}} cargo +nightly run --example crud_rocket_mysql
```


## Contribution
My near term goal is to support more web frameworks and databases. However I would like to stabilise the API first. So you are welcome to play around and test it. **Don't use it in production yet**. Comments, bug fixes and quality improvements are welcome. For features please hold on.

## Related projects
[Diesel](http://diesel.rs/) is an  ORM for Rust
[GraphQL](https://github.com/graphql-rust) can query data.

## Background

I developped the initial Toql language about 5 years ago for a web project. I have refined it since then and you can see it in action on [www.schoolsheet.com](www.schoolsheet.com), a webservice I created in Java for my teaching. The Rust implementation is must faster though ;)


## License

Toql is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

