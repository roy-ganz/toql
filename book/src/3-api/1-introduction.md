# The Toql API

Toql relies on backends to handle database differences. 
These backends implement the `ToqlApi` trait 
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
| MySQL   | toql_mysql_async| mysql_async |

For MySQL add this to your `cargo.toml`:

```toml
[dependency]
toql = "0.3"
toql_mysql_async = "0.3"
```

You must add `toql ` together with the backend crate. The backend crate then depends on a suitable version of the driver crate.
Normally there is no need to access the driver crate. However I you really must, the backend crate re-exports the driver crate. 
For `toql_mysql_async` the driver crate can be accessed through `toql_mysql_async::mysql_async`.

With these two dependencies you can get the backend in your code. Notice that the backend takes 
a database connection and a cache object to hold the database mapping.

```rust
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

There are three ways to feed in aux params: 
- You can put them in the context and they will be available as long as the
  backend object lives
- You can also ship them with a query and they will be available only for that query
- You can map aux params to a field. Used to configure [field handlers](../4-derive/5-field-handlers.md).

Here how to put them in the context:

```rust
use mysql_async::MySql;
use toql_mysql_async::prelude::MySqlAsync;
use toql::prelude::{Cache, ContextBuilder};
use std::collections::HashMap;


let pool = mysql_async::Pool::new(database_url);
let mut conn = pool.get_conn().await?;

let mut p = HashMap::new();
p.insert("page_limit".into(), 200.into());

let context = ContextBuilder::new().with_aux_params(p).build();
let cache = Cache::new();
let toql = MySqlAsync::with_context(&mut conn, &cache, context);
```

Beside aux params `ContextBuilder` allows you 
  - to choose an alias format (`user.id`, `us1.id`, `t0.id`, ...)
  - set the roles for [access control](../4-derive/16-roles.md)


 ```rust
 use toql::prelude::{ContextBuilder, AliasFormat};
 use std::collections::HashSet;

 let mut roles = HashSet::new();
 roles.insert("teacher", "admin");

  let context = ContextBuilder::new()
    .with_alias(AliasFormat::Tiny)
    .with_roles(roles)
    .build();
 ```













