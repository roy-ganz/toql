## Inserts

There are two insert functions: `insert_one`, and `insert_many`. 

The are used like so:

```
use toql::prelude::{ToqlApi, paths};

let u = User {id:0, title: "hello".to_string(), adress: None};

toql.insert_one(&mut u, paths!(top)).await?;
toql.insert_one(&mut u, paths!(User, "")).await?;

toql.insert_many(&[&mut u], paths!(top)).await?;
```

In the example above the first `insert_one` will insert `u` into the database, 
load back the generated id and sets it on `u`. 
The second `insert_one` makes a copy of `u` and again refreshes its `id` field.

Optional fields that are `None` will insert the default value of the database. See XX for details.


### The paths! macro
The `paths!` macro compiles a path list. Any invalid paths show up at compile time.

The insert functions will then insert all referenced joins and merges from that list.
A path list like `paths!(User, "books_publisher")` will insert 
- the base entity, 
- the joined publisher
- all merged books

If you only want to insert a publisher, then you must call insert for a publisher.

### Partial tables
It is possible to split up a table into multiple tables. See Mapping XXX for details.

If a path in the path list refers to a struct that contains joins marked as `partial table` then these
joins will also be inserted. There is no need teo mention these dependencies in the path list.

### Key dependencies
The order of SQL execution is based on the key dependencies. 

After the SQL insert on the database, Toql will load back the generated ids 
to update the primary key and foreign keys of the inserted struct.





 







