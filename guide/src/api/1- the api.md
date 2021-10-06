# The Toql Api

Toql relies on backends to handle database differences. 
These backends implement a common trait, the `ToqlApi`, 
which serves as an entry point for any high level function.
The backends then use the Toql library to do their job.

This chapter explains how to use the `ToqlApi` trait. 
Notice that you must derive your structs before you can load or modify them 
with the ToqlApi. See the mapping chapter for details.


It is also possible to write database independend code. This is described in the last chapter.

## Creating the backend
To use the `ToqlApi` functions you need a backend for your database. 
Since Toql is a async ORM, it only supports the async database drivers.

Currently the following backends are available

|Database | Toql Crate     | Driver Crate|
|---------|----------------|-------------|
| MySql   | toql_mysql_async| mysql_async |

To use MySql you need to add these dependencies in `cargo.toml`:

```
[dependency]
toql = "0.3"
mysql_async = "0.20"
toql_mysql_async = "0.3"
```

Then you can get the backend in your code.

```
use mysql_async::MySql;
use toql_mysql_async::prelude::MySqlAsync;

let mut conn = Mysql::new();
let toql = MySqlAsync::from(&mut conn);
```

Often you may want to feed configuration or authentication values into your Sql.
This is done through so called auxiliary parameters (aux params):

```
use mysql_async::MySql;
use toql_mysql_async::prelude::MySqlAsync;
use toql::prelude::ContextBuilder;

let mut conn = Mysql::new();
let mut p = HashMap::new();
p.insert("page_limit".into(), 200.into());

let context = ContextBuilder::new().set_aux_params(p).build();
let toql = MySqlAsync::with_context(&mut conn, context);
```

Note that there are two places to feed in aux params: 
- You can add them in the context and they will be available as long as the
  backend object lives
- You can also add them to the query and the will be available only for that query









