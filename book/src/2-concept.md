# Concept

Toql is a ORM that aims to boost your developer comfort and speed when working with databases.

To use it you must derive `Toql` for all structs that represent a table in your database:
- A field in those structs represents either a columns, an SQL expression or a
relationship to one or many tables.
- The field also determines the field name or in case of a relationship the path name in the [Toql query](5-query-language/1-introduction.md)

A struct may map only some columns of a table and also multiple structs may refer to the same table. Structs are rather 'views' to a table.

A derived struct can then be inserted, updated, deleted and loaded from your database. To do that you must call the [Toql API functions](3-api/1-introduction.md) with a query string or just a list of fields or paths.

Here the typical flow in a web environment:
1. A web client sends a Toql query to the REST Server.
2. The server uses Toql to parse the query and to create SQL statements.
3. Toql sends the SQL to the database
4. then deserializes the resulting rows into Rust structs.
5. The server sends these structs to the client.

## Quickstart
There is full featured [REST server](https://github.com/roy-ganz/todo_rotomy) based on Rocket, Toql and MySQL. It can be used as a playground or starting point for own projects.


