# Toql Derive reference

The derive provides struct level attributes and field level attributes. Here a list of all available attributes:

## Attributes for structs

Attribute | Description                             | Example / Remark
---- |---| ---|
tables  |   Table renaming scheme for struct and joins. |  `CamelCase`, `snake_case`, `SHOUTY_SNAKE_CASE` or `mixedCase`
columns        | Column renaming scheme. |
table | Table name for a struct or join. | table ="User" on struct `NewUser` will access table `User`
skip_query | No query methods are generated. | 
skip_query_builder | No field methods are generated. |  No `User::fields.id()`.
skip_indelup |No insert, delete and update methods are generated. |

## Attributes for fields  

Attribute | Description | Example / Remark
---- |---| ---|
delup_key | Field used as key for delete and update functions. | For composite keys use multiple times.
skip_inup | No insert, update for this field. | Use for auto increment columns or columns calculated from database triggers.
sql       | Maps field to SQL expression instead of table column. To insert the table alias use two dots. Skipped for insert, update. | `sql ="SELECT COUNT (*) FROM Message m WHERE m.user_id = ..id"`
sql_join  | Loads a single struct with an sql join, where self and other defines columns with same values.    | For composite keys use multiple `sql_join`.  `sql_join( self="column_name_on_this_table", other="column_name_on_joined_table", on="friend.best = true")`
merge     | Loads a dependend Vec<>. Merges run an additional SELECT statemen. self and other define struct fields with same values.  | `merge(self="rust_field_name_in_this_struct", other="rust_field_name_on_other_struct")` For composite fields use multiple `merge`.
ignore_wildcard | No selection for `**` and `*`| 
alias | Builds sql_join with this alias. | 
table | Joins or merges on this table. | 
role | Field only accessable for queries with this role. Multiple use requires multiple roles. | `role="admin", role= "superadmin"`
