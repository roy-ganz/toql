# The Toql Api

Toql relies on backends to handle database differences. 
These backends implement a common trait, the `ToqlApi`, 
which serves as an entry point for any high level function.
The backends then use the Toql library to do their job.

This chapter explains how to use the `ToqlApi` trait. 
Notice that you must derive your types, before you can load or alter them 
with the ToqlApi. See the mapping chapter for details.


It is also possible to write backend agnostic code. See the next chapter for details on this.

## Creating the backend
To use the Toql functions you need a backend for your database. 
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

Often you may want to feed in configuration or authentication values into your Sql.
To provide auxiliary parameters (aux params) do this:

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
  toql object lives
- You can also add them to the query and the will be available only for that query

What you do depends on your use case: Typically configuration values are put into the context
and authentication values are provided with the query. 
But that's up to you, the Sql building stage in Toql will combine all of them anyway.








