# The Toql Api

Toql relies on backends to handle database differences. 
These backends implement a common trait, the `ToqlApi`, 
which serves as an entry point for any high level function.
The backends internally then use the Toql library to do their job.

This chapter explains how to use the `ToqlApi` trait. 
Notice that you must derive your structs before you can load or modify them 
with the `ToqlApi`. See the [derive chapter](../4-derive/1-introduction.md) for details.


The common `ToqlApi` trait makes it also possible to write database independend code. This is described [here](8-backend-independence.md).

## Creating the backend
To use the `ToqlApi` functions you need a Toql backend and the driver for your database. 

Currently the following backends are available

|Database | Backend Crate     | Driver Crate|
|---------|----------------|-------------|
| MySql   | toql_mysql_async| mysql_async |

For MySql add this to your `cargo.toml`:

```
[dependency]
toql = "0.3"
mysql_async = "0.20"
toql_mysql_async = "0.3"
```

Then you can get the backend in your code. Notice that the backend takes 
a database connection and a cache object to keep database schema information.

```
use mysql_async::MySql;
use toql_mysql_async::prelude::MySqlAsync;
use toql::prelude::Cache;

let pool = mysql_async::Pool::new(database_url);
let mut conn = pool.get_conn().await?;

let cache = Cache::new();

let toql = MySqlAsync::from(&mut conn, &cache);
```

In a bigger project you may want to feed configuration or authentication values into your SQL.
This is done through so called auxiliary parameters (aux params).

There are two ways to feed in aux params: 
- You can put them in the context and they will be available as long as the
  backend object lives
- You can also ship them with a query and they will be available only for that query

Here how to put them in the context:

```
use mysql_async::MySql;
use toql_mysql_async::prelude::MySqlAsync;
use toql::prelude::{Cache, ContextBuilder};


let pool = mysql_async::Pool::new(database_url);
let mut conn = pool.get_conn().await?;

let mut p = HashMap::new();
p.insert("page_limit".into(), 200.into());

let context = ContextBuilder::new().set_aux_params(p).build();
let cache = Cache::new();
let toql = MySqlAsync::with_context(&mut conn, &cache, context);
```












