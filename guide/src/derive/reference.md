# Toql Derive reference

The derive provides struct level attributes and field level attributes. Here a list of all available attributes.

## Attributes for structs

Attribute | Description                             | Example / Remark
---- |---| ---|
tables  |   Defines for struct and joins how table names are generated. |  Possible values are `CamelCase`, `snake_case`, `SHOUTY_SNAKE_CASE` and `mixedCase`
columns        | Same as attribute `tables` but for columns. |
table | Sets the table name for a struct or join. | table ="User" on struct `NewUser` will access table `User`
skip_query | Derive does not generate query functionality for the struct. | 
skip_query_builder | Derive does not generate field methods. |  No `User::fields.id()`.
skip_indelup | Derive does not generate insert, delete and update functionality. |

## Attributes for fields  

Attribute | Description | Example / Remark
---- |---| ---|
delup_key | Field used as key for delete and update functions. For composite keys use multiple times. |
skip_inup | No insert, update for this field. | Use for auto increment columns or columns calculated from database triggers.
sql       | Maps field to SQL expression instead of table column. To insert the table alias use two dots. Skipped for insert, update. | `sql ="SELECT COUNT (*) FROM Message m WHERE m.user_id = ..id"`
sql_join  | Loads a single struct with an sql join, where self and other defines columns with same values. For composite keys use multiple `sql_join`.    | `sql_join( self="friend_id", other="friend.id", on="friend.best = true")`If _self_ is omitted it will be created from variable name. 
merge     | Loads a dependend Vec<>. Merges run an additional SELECT statemen. self and other define struct fields with same values. For composite fields use multiple merges | `merge(self="id", other="user_id")`
ignore_wildcard | No selection for `**` and `*`| 
alias | Builds sql_join with this alias. | 
table | Joins or merges on this table. | 
role | Field only accessable for queries with this role. Multiple use requires multiple roles. | `role="admin", role= "superadmin"`
