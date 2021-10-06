## Updates

There are two update functions: `delete_one`, and `delete_many`. 

Bothe delete functions take a predicate and delete all rows that match the predicate. 

`delete_one` takes a key or entity. It will turn the key into q query see 3- query 
and delete the row that matches the predicate.

`delete_many` builds a predicate from the filters of the `Query` argument. All selections in the Query are ignored.

Notice that delete does not do any cascading yb itself. It just deletes the base type. 
To cascade your deletes you must configure your database relations 
and tell the database what to do with your joined rows: Delete them too or just set the foreign key to NULL.

For MySql see https://dev.mysql.com/doc/refman/8.0/en/create-table-foreign-keys.html
For Postgres see https://www.postgresql.org/docs/8.2/ddl-constraints.html#DDL-CONSTRAINTS-FK


```
use toql::prelude::ToqlApi;

toql.delete_one(UserKey::from(5))?;

let u = User {id: 5};
toql.delete_one(u).await?;

toql.delete_many(query!(User, "id eq 5")).await?;
```
