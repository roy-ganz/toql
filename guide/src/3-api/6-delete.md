## Deletes

There are two update functions: `delete_one`, and `delete_many`. 

Bothe delete functions take a predicate and delete all rows that match the predicate. 

`delete_one` takes a key or entity. It will build a filter predicate from that and delete the row that matches the predicate.

`delete_many` builds a predicate from the filters of the `Query` argument. All selections in the query are ignored.

```
use toql::prelude::ToqlApi;

toql.delete_one(UserKey::from(5))?;

let u = User {id: 5};
toql.delete_one(u).await?;

toql.delete_many(query!(User, "id eq 5")).await?;
```

### Cascading
`delete` does not do any cascading by itself. It just deletes rows from a single table. 
To cascade your deletes you must configure your database relations 
and tell the database what to do with your joined rows: Delete them too or just set the foreign key to NULL.

Check the manual for [MySql](https://dev.mysql.com/doc/refman/8.0/en/create-table-foreign-keys.html) or for [Postgres](https://www.postgresql.org/docs/8.2/ddl-constraints.html#DDL-CONSTRAINTS-FK)
.