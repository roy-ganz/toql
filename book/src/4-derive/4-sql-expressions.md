# SQL expressions
Toql is an SQL friendly ORM. Instead of mapping a struct field to a column you can also map it
to a raw SQL expression. There are small syntax enhancements to work with aliases and auxiliary parameters.

#### Alias axample

```rust
#[derive(Toql)]
struct User {
    #[toql(key)]
    id: u64,

    #[toql(sql="(SELECT COUNT(*) FROM Books b WHERE b.author_id = ..id)")]
    number_of_book:u64
}
```

Notice the `..` ! This special alias will be replaced with the alias crated for _User_.
The generated SELECT might look like this:

```
SELECT t0.id, (SELECT COUNT(*) FROM Books WHERE author_id = t0.id) FROM User t0
```

To use aux params in a SQL query use the `<param_name>` syntax. 

#### Aux params example

```rust
#[derive(Toql)]
struct User {
    #[toql(key)]
    id: u64,

    #[toql(sql="(SELECT <page_limit>)")]
    page_limit:u64

     #[toql(sql="(SELECT COUNT(*) FROM Films f WHERE f.age >= <age>)")]
    age_rated_films:u64
}
```
In the example *page_limit* might come from a server configuration. 
It would typically be put in the [context](../3-api/1-introduction.md) and can be used in SQL expressions.

The aux param *age* might be taken from the authorisation token and put as an aux param into the context or query. 
Here it restricts the number of films.

## Other uses of raw SQL
There are other places you can use raw SQL:
 - [Predicates](10-predicates.md)
 - [Custom Merge](4-derive/5-merges)