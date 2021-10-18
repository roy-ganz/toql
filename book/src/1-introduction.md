![Bollard by the Sea by Gábor Szakács (Public Domain)](bollard-at-the-port.gif)

# Toql - A friendly and productive ORM

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

This guide will explain you how to use Toql in your own project.

Toql is free and open source software, distributed under a dual license of MIT and Apache. The code is available on [Github](https://www.github.com/roy-ganz/toql). Check out the API for technical details.

### Trivia

- Toql is pronounced *to-cue-ell* with o as in object.
- Toql stands for _Transfer Object Query Language_ and refers to the query language that is unique to this ORM. In a sense though it's a missleading name, because Toql together with Serde effectively avoid the need for data transfer objects (DTO): you pass your model directly.
- The project's mascot is a bollard, because Toql pronounced in allemanic / swiss german sounds like _Toggel_: 
A funny word that can colloquially be used for bollards.

