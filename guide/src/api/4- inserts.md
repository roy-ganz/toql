## Inserts

There are two insert functions: `insert_one`, and `insert_many`. 

The are used like so:

```
use toql::prelude::{ToqlApi, paths};

let u = User {id:0, title: "hello".to_string()};

toql.insert_one(&mut u, paths!(top))?;
toql.insert_one(&mut u, paths!(User, ""))?;

toql.insert_many(&[&mut u], paths!(top))?;
```

In the example above the first `insert_one` will insert `u` into the database, 
load back the generated id and sets it on `u`. 
The second `insert_one` makes a copy of `u` and again refreshes its `id` field.

### The paths! macro
The `paths!` macro will compile a path list and any invalid paths wil show up at compile time.

insert order





