
## The query argument
All load functions need a query argument, but how is this build?

The recommended way is to use the `query!` macro.

Alternatives are 
- to use the query builder, see XXX
- or to parse a string and deal with errors at runtime, see XX

This chapter does not explain the Toql query language itself, see chapter XXX to learn about that.


### The query! macro 
The `query!` macro will compile the provided string into Rust code. Any syntax mistakes, wrong path or field names show up 
as compiler errors! 

Queries are typesafe, so `query!` takes a type and a query expression. See here:

```
use toql::prelude::query;
let user_id = 5;
let q = query!(User, "*, id eq ?",  user_id);
```
 
To include query parameters just insert a question mark in the query string and provide the parameter after the string. 

In the example above it would also be possible to put the number 5 directly into the query string, since it's a constant. 
The resulting SQL would be the same, as Toql extracts the parameter in either case to prevent Sql injections.

The Toql query only works with numbers and strings, see `SqlArg`. 
However this is not be a problem: Since database columns have a type, the database is able convert a string or number into its column type.

It's also possible to include other queries into a query. Consider this:

```
use toql::prelude::query;
let q1 = query!(User, "id eq ?", 5);
let q = query!(User, "*, {}", q1);
```

Here we include the query `q1` into `q`. Since queries are typesafe, so you can only include queries of the same type.

### Working with keys

There are situations with entities that may have composite keys when it's easier to work with keys.
(Keys are automatically derived from the `Toql` derive and are located where the struct is.)

With a single key, this is possible
```
use toql::prelude::query;

let k = UserKey::from(5);
let q1 = query!(User, "id eq ?", k);
let q2 = query!(User, "*, {}", k.to_query());
let q3 = query!(User, "*, {}", k);
```

With multiple keys, you can do this:
```
use toql::prelude::{query, Query};

let ks = vec![UserKey::from(1), UserKey::from(2)];

let qk = ks.iter().collect::<Query<_>>();
let q4 = query!(User, "*, {}", qk);
```

Or with mutiple entities:

```
use toql::prelude::{query, MapKey, Query};

let es = vec![User{id:1}, User{id:2}];

let qk = es.iter().map_key().collect::<Query<_>>();
let q5 = query!(User, "*, {}", qk);
```

Both `q4` and`q5` end up the same.

### Into<Query>

In the example above the query `q3` is build with a `UserKey`. This is possible because `UserKey` implements `Into<Query<User>>`.
You can also implement this trait for you own types. Let's assume a book category.

#### Usecase 1: Adding an enum filter to the query
```
enum BookCategory {
    Novel,
    Cartoon
}
impl Into<Query<Book> for BookCategory {
    pub fn info(&self) {
       query!(Book, "category eq ?", 
       match self {
        Novel => "NOVEL",
        Cartoon => "CARTOON"    
       })
    }
}

```

Now use it like this
```
let q = query!(Book, "*, {}", BookCategory::Novel);
```

#### Usecase 2: Adding an authorisation filter to the query


```
use toql::prelude::{QueryWith, Query, Field}
struct Auth {
    user_id: u64
}
impl Into<Query<Book>> for Auth {
    pub fn into(self) -> Query<Book> {
        query.and(Book::fields().author_id().eq(self.user_id))
    }
}
```

You may want trade typesafety for more flexibility. See the example above again, this time with the `Field` type.

```
use toql::prelude::{ Query, Field}
struct Auth {
    author_id: u64
}
impl<T> Into<Query<T>> for Auth {
    pub fn into(&self) -> Query<T>{
        Query::from(Field::from("authorId").eq(self.author_id))
    }
}
```
Wrong field names in `Field::from` do not show up at compile time, but at runtime.

You can use both examples like so

```
use toql::prelude::query;
let auth = Auth {author: 5};
let q = query!(Book, "*, {}", auth);
```


### The QueryWith Trait

The `query!` macro produces a `Query` type and can therefore further be altered using all methods from that type.
One interesting method is `with`. It can be implemented for any custom type to enhance the query. 
This is more powerful than `Into<Query>` because you can also access auxiliary parameters.

Aux params can be used in SQL expressions. See the chapter on mapping XX for more information.

```
struct Config {
    limit_pages: u64
}
impl QueryWith for Config {
    pub fn with(&self, query: Query<T>) {
        query.aux_param("limit_pages", self.limit_pages)
    }
}
```

This can now be used like so:

```
use toql::prelude::query;
let config = Config {limit_pages: 200};
let k = UserKey::from(5);
let q = query!(User, "*, {}", k.to_query()).with(config);
```


### Parsing queries
Use the query parser to turn a string into a `Query` type. 
Only syntax errors will returns as errors, 
wrong field names or paths will be rejected later when using the query.

```
use toql::prelude::Parser;

let s = "*, id eq 5";

let q = QueryParser::parse::<User>(s).unwrap();
```






