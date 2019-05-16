# Toql (_Transfer Object Query Language_)

This guide will explain you how to use Toql to query and modify data from a database.

Toql is free and open source software, distributed under a dual license of MIT and Apache. The code is available on [Github](www.github.com/roy-ganz/toql). Check out the API for technical details.

## Getting started

This book is split into several sections, with this introduction being the first. The others are:

* [Concept](concept.md) - The overall concept of Toql.
* [Query Language](query-language/introduction.md) - How queries look like.
* [Toql Derive](derive/introduction.md) - Let the derive do all the work!

## Project structure

Toql is made of
* A __query parser__ to parse query string from web clients.
* A __query builder__ to modify or create queries on the fly.
* A __SQL mapper__ that translates Toql fields into SQL columns or expressions.
* A __SQL builder__ to turn a query into SQL with the help of the mapper.
* A __Toql derive__ that generates mappings of structs, functions to handle dependencies and helper functions.
* __3rd party integration__  to work well together with Rocket and MySQL.
 
 ## Background
 I developed Toql about 10 years ago for a web project. I have refined it since then and it can be seen in action
 on my other website [www.schoolsheet.com](http://www.schoolsheet.com). I started the Toql project to learn Rust.






