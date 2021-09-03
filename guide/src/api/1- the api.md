**The Toql Api**

Toql relies on backends to handle database differences. 
This backends implement also a common trait, the ToqlApi, 
which serves as an entry point for all users to call any function.
The functions in the ToqlApi then call functions from the Toql library to do their job.
This chapter explains how tu use the ToqlApi.

It is also possible to write backend agnostic code. See net chapter for details on this.

## Creating the backend
To use the toql functions you need a backend for your database. 
Since Toql is a async ORM, it only supports the async databse drivers.
Currently the following backends are available

|Database | Toql Crate     | Driver Crate|
|---------|----------------|-------------|
| MySql   | toql_mysql_async| mysql_async |

To use MySql you need in add these dependencies in `cargo.toml`:
```
[dependency]
toql = "0.3"
mysql_async = "0.20"
toql_mysql_async = "0.3"
```

Then you can get the backend in your code.

```
let mut conn = Mysql::new();
let toql = MySqlAsync::from(&mut conn);
```

Often you may want to feed in configuration or authentication values into your Sql.
Do this then:

```
let mut conn = Mysql::new();
let p = 
let context = ContextBuilder::new().set_aux_params(p).build();
let toql = MySqlAsync::with_context(&mut conn, context);
```

Note there are two places to feed in aux params: 
- You can add them in the context and they will be available as long as the
  toql object lives
- You can also add them to the query and the will be available only for that query

What you do depends on your usage: Typically configuration values are given for the context
where as authenicaltion values are given with the query. 
But it's up to you, the Sql building stage in Toql will combine all of them anyway.



## Loading

There are three loading functions: load_one, load_many and load_page.
All loading functions will select, filter and order columns or expressions 
acccording to the query argumnet and the type mapping, see XXX . If needed, the load functions issue multiple select
statements on your database and merge the results.

If you expect exactly one result, use load_one

```
    use toql::prelude::{query, ToqlApi};

    let toql = ...
    let q = query!(...);
    let u = toql.load_one(q).await?;
```
The function will return NotFound if no row matched the query filter or NotUnique if too many rows matched.
To load zero or one row use load_page, see below.

Similarly, if you need to load multiple rows:


```
    use toql::prelude::{query, ToqlApi};

    let toql = ...
    let q = query!(...);
    let u = toql.load_many(q).await?;
```

load_many returns a Vec<> with deserialized rows. 
The Vec will be empty, if no row matched the filter criteria.

load_page allows you to select a page with a starting point and a certain length. 
It returns a tuple of a Vec and count infromation.

The count information is either None for an uncounted page, 
or contains count statistics that are needed for typical pagers in web apps, see below.
(After all Toql was initially created to serve web pages)

In case you want to load the first 10 -or less- rows do this

```
    use toql::prelude::{query, ToqlApi, Page};

    let toql = ...
    let q = query!(...);
    let (u, _) = toql.load_page(q, Page::Uncounted(0, 10)).await?;
```

To serve a webpage, you may also want to include count informations.

```
    use toql::prelude::{query, ToqlApi, Page};

    let toql = ...
    let q = query!(...);
    let (u, c) = toql.load_page(q, Page::Counted(0, 10)).await?;
```

The code is almost the same, but the altered page argument will issue two more select statements
to return the filtered page length and the unfiltered page length. Let's see what those are:

Suppose you have a table with books. The books have an id, a title and an author_id.

Books:
------
id, title, author_id
1, The world of foo, 1
2, The world of bar, 1
3, The world of baz, 1
4, My life with 42, 1
5, Plants And Trees, 2


Let's assume we have a webpage that contains a pager with page size 2 and a pager filter. 
The author wants to see all books that contain the word 'world'. What will he get?
 - The first two rows (id 1, id 2).
 - The filtered page count of 3, because 3 rows match the filter criteria. 
   The pager can now calculate the number of pages: ceil(3 / 2) = 2
 - The unfiltered page count of 4. The author knows now that with a different filter query, he could
   get at most 4 rows back.
 
 In practice the unfiltered page count is not so straight forward to select. 
 Toql needs to decide, which filters to ignore and which to consider, 
 when building the count sql statement.
 Toql considers only filters from the query, whose fields are listed in the special count selection.
 
### Bilding the query argument
All load functions need a query argument, but how is this build?
The recommended way is to use the `query!` macro.
This macro will compile the provided string into Rust code. Any syntax mistakes or wrong path and field names show up 
as compiler errors! 
To learn about Toqls unique query language see it's dedicated chapter. Here we just have a look at the query! macro.

Here is an example to load all fields from type User with id 5.
```
use toql::prelude::query;

let q = query!(User, "*, id eq ?", 5);
```
 
To include parameters just insert a question mark in the query string and provide the parameter after the string. 
The Toql query only works with a limited type of parameters (numbers and strings), see `SqlArg`. 
However this should not be a problem: Since database columns have a type, e.g datetime, 
the database is able convert a string or number into its column type.

It's also possible to include other querries into a query. Consider this:

```
use toql::prelude::query;
let q1 = query!(User, "id eq ?", 5);
let q = query!(User, "*, {}", q1);
```
Here we include the query q1 into q. Notice that queries are typesafe, so you can only include queries of the same type.

Including a query is also useful when you work with keys, since you can turn a key into a filter statement. See here

```
use toql::prelude::query;
let k = UserKey::from(5);
let q = query!(User, "*, {}", k.to_query());
```

or for a list of keys:
```
use toql::prelude::query;
let k = vec![UserKey::from(5), UserKey(10)];
let q = query!(User, "*, {}", k.to_query());
```

The query macro produces a Query type and can therefore further altered using all methods from that type.
On interesting method is .with. If can be implemented for any custom Type and can enhance the query.

#### Usecase 1: Adding config values as aux params to the query
Aux params can be used in Sql expressions. See the Mapping chapter  for more information.

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


#### Usecase 2: Adding an authorisation filter to the query
```
use toql::prelude::{QueryWith, Query, Field}
struct Auth {
    author_id: u64
}
impl<T> QueryWith<T> for Auth {
    pub fn with(&self, query: Query<T>) {
        query.and(Field::from("authorId").eq(self.author_id))
    }
}
```

Notice the `Field::from` method. Queries are always typed, however sometimes some
hackery is just too convenient to be missed out and `Field` just allows that. 

If you are into stricter type safety, you can do this

```
use toql::prelude::{QueryWith, Query, Field}
struct Auth {
    user_id: u64
}
impl QueryWith<Book> for Auth {
    pub fn with(&self, query: Query<T>) {
        query.and(Book::fields().author_id().eq(self.user_id))
    }
}
```


Now you can use it like above
```
use toql::prelude::query;
let auth = Auth {author: 5};
let k = UserKey::from(5);
let q = query!(User, "*").with(auth);
```









