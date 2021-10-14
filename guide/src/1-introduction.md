# Toql (_Transfer Object Query Language_)

Toql (pronounced *To-cue-ell*) is an ORM for async databases that features
- Translation between Rust structs and database tables.
- Can load and modify nested structs.
- Has a unique dead simple query language.
- Different table aliases from long and readable to tiny and fast.
- Prepared statements against SQL Injection
- Support for raw SQL for full database power
- Support for role based access
- Highly customizable through user defined parameters, query functions, field handlers, etc. 
- Compile time safety for queries, fields and path names.
- No unsafe Rust code
- Tested on real world scenario

This guide will explain you how to use Toql in your own project.

Toql is free and open source software, distributed under a dual license of MIT and Apache. The code is available on [Github](https://www.github.com/roy-ganz/toql). Check out the API for technical details.


