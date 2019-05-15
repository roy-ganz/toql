# Toql (_Transfer Object Query Language_)

This guide will explain you how to use Toql to query and modify data from a database.

Toql is free and open source software, distributed under a dual license of MIT and Apache. The code is available on [Github](www.github.com/roy-ganz/toql). Check out the API for technical details.

## Getting started

This book is split into several sections, with this introduction being the first. The others are:

* [Concept](concept.md) - The overall concept of Toql.
* [Query Language](query-language/introduction.md) - How queries look like.
* [Toql Derive](derive/introduction.md) - Let the derive do all the work!

## Features

Toql _Transfer Object Query Language_ is a set of crates that turns a query string into SQL to retrieve data records. 
It is useful for web clients to get database records from a REST interface. 

Toql
 - can query, insert, update and delete single and multiple database records.
 - handles dependencies in queries through SQL joins and merges. Cool!
 - is fast, beause the mapper is only created once and than reused.
 - has high level functions for speed and low level functions for edge cases.
 
 ## Background
 I developped Toql about 10 years ago for a web project. I have refined it since then and the current Java version can be seen in action on my other website [www.schoolsheet.com](http://www.schoolsheet.com). I started the Toql project to learn Rust.






