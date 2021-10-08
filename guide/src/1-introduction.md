# Toql (_Transfer Object Query Language_)

Toql (pronounced *To-cue-ell*) is an ORM for async databases that features
- Translation between Rust structs and database tables.
- Can load and modify nested structs.
- Has a unique dead simple query language.
- Choose from long to tiny  table aliases  for your Sql.
- Prepared statements against Sql Injection
- Map struct fields to raw Sql for full database power
- Support for role based access
- Highly customizable through user defined parameters, query functions, field handlers, etc. 
- Compile time safety for queries, fields and path names.
- Query builder cache
- No unsafe Rust code
- Tested on real world scenarios

This guide will explain you how to use Toql in your own project.

Toql is free and open source software, distributed under a dual license of MIT and Apache. The code is available on [Github](https://www.github.com/roy-ganz/toql). Check out the API for technical details.

## Available Sections

This guide is split into several sections, with this introduction being the first. The others are:

* [Concept](2-concept.md) - The overall concept of Toql.
* [Toql Api](3-api/1-toql-api.md) - How to use the library.
* [Toql Derive](4-derive/1-introduction.md) - Map Rust structs to database tables and query fields.
* [The Toql query Language](5-query-language/1-introduction.md) - How to build queries.
* [Appendix](6-appendix/1-overview)  - Different tipps and tricks