# Toql

### Description
Toql *Transfer object query language* is a query language to build SQL statements to retrieve filtered, ordered and indiviually selected columns from a database.

Toql turns this
```toql
id, (+age eq 16; age eq 18), adress_street
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
Check out the [CRUD example](https://github.com/roy-ganz/toql/blob/master/examples/rocket_mysql/main.rs). There is also a [guide](https://github.com/roy-ganz/toql/blob/master/guide/src/introduction.md) and the [API documentation](https://docs.rs/toql/0.1.4/toql/).

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
toql ="0.1"
```

## Project

Toql is made of

* A __query parser__ to parse query string from web clients.
* A __query builder__ to modify or create queries on the fly.
* A __SQL mapper__, that translates Toql fields into SQL columns or expressions.
* A __SQL builder__ to turn a query into SQL with the help of the mapper.
* A __Toql derive__ that generates mappings of structs, functions to handle dependencies and helper functions.
* __3rd party integration__  to work well together with Rocket and MySQL.


## Contribution
My near term goal is to support for more web frameworks and databases. However I would like to stabilise the API first. So you are welcome to play around and test it (**don't use it in production yet**). Comments, bug fixes and quality improvements are welcome. For features please hold on.

## Other database projects
- [Diesel](http://diesel.rs/)
- [GraphQL](https://github.com/graphql-rust)


## Background
I have developed the initial Toql language about 7 years ago for a web project. I have refined it since then and you can see it in action on [www.schoolsheet.com](http://www.schoolsheet.com), a web service I created in Java for my teaching. The Rust implementation is must faster though ;)


## License

Toql is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

