# Toql (_Transfer Object Query Language_)

This guide will explain you how to use Toql to query and modify data from a Database.

Toql is free and open source software, distributed under a dual license of MIT and Apache. The code is available on [Github](https://github.com/roy-ganz/toql/blob/master/guide/src/introduction.md)

## Getting started

This book is split into several sections, with this introduction being the first. The others are:

* [Concept](concept.md) - The overall concept of Toql.
* [Query Language](query-langugae.md) - How queries look like.
* [Query Builder](query-builder.md) - How queries can be programmed with typesafety. 
* [Toql Derive](toql-derive.md) - Let the derive do all the work!
* [SQL Mapper](sql-mapper) - How to fine tune and tweak the derives output.
* [Rocket](rocket-integration.md) - Use rocket to provide your REST API.
* [MySQL](mysql-integration.md) - Use MySQL as a Database.

Toql _Transfer Object Query Language_ is a library that turns a query string into SQL to retrieve data records. 
It is useful for web clients to get database records from a REST interface. 

Toql
 - can query, insert, update and delete single and multiple database records.
 - handles dependencies in queries through SQL joins and merges.
 - is fast, beause the mapper is only created once and than reused.
 - has high level and low level functionality for tweaking.
 

The project consists of
* A __query parser__ to create a query from a string (from web clients).
* A __query builder__ to create or change queries programatically
* An __SQL mapper__ to translates toql fields into SQL columns or expressions.
* A __SQL builder__ to turn a query into SQL using the mapper.
* A __toql derive__ to generates all boiler plate for structs.
* __3rd party integration __  to work well together with Rocket and MySQL. (More planned).



