## Writing functions

In bigger projects you need to structure your code with functions. 

There as two common ways, each with different tradeoffs
- Pass the database driver to the function
- Pass ToqlApi to the function

### Passing the database

If you decide to pass the database you give up on database independence, but less trait bounds are needed:

For MySQL this looks like this:

``rust
use toql::prelude::ToqlApi;
use toql_mysql_async::prelude::{MySqlAsync, Queryable};
fn do_stuff<C>(toql: &mut MySqlAsync<'_,C>) 
where C:Queryable -> Resulty
{
    let q = query!(...)
    let users = toql.load_many(&q).await?;
    toql.insert_many(users, paths!(top)).await?;
    toql.update_many(users, fields!(top)).await?;
    toql.delete_many(q).await?;
}
```
The `Queryable` trait makes the `MySqlAsync` work with a connection or a transaction.



## Database independed functions

It's also possible to pass a struct that implements `ToqlApi`. 
However this requires more trait bounds to satisfy the bounds on `ToqlApi`.
Unfortunately rust Rust compiler has a problem with [associated type bounds](https://rust-lang.github.io/rfcs/2289-associated-type-bounds.html), so it looks more complicated than it had to be.








