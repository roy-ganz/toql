# Toql Derive reference

The derive provides struct level attributes and field level attributes. Here a list of all available attributes:

## Attributes for structs

Attribute | Description                             | Example / Remark
---- |---| ---|
tables  |   Table renaming scheme for struct and joins |  `CamelCase`, `snake_case`, `SHOUTY_SNAKE_CASE` or `mixedCase`
columns        | Column renaming scheme |
table | Table name for a struct or join | `table ="User"` on struct `NewUser` will access table `User`
skip_query | No query methods  | 
skip_query_builder | No field methods |  No `User::fields.id()`.
skip_indelup |No insert, delete and update methods |

## Attributes for fields  

Attribute | Description | Example / Remark
---- |---| ---|
delup_key | Field used as key by delete and update methods | For composite keys use multiple times.
skip_inup | No insert, update for this field | Use for auto increment columns or columns calculated from database triggers.
sql       | Field mapped to SQL expression instead of table column | Insert the table alias with two dots: `sql ="SELECT COUNT (*) FROM Message m WHERE m.user_id = ..id"`. Skipped for insert, update
sql_join  | Required for fields that are structs   | `sql join` needs column names in `self` and `other`, with `on` an extra sql condition can be given: `sql_join( self="column_name_on_this_table", other="column_name_on_joined_table", on="friend.best = true")`. For composite keys use multiple `sql_join`.
merge     | Required for fields that are Vec<> | `merge` needs struct field names in `self` and `other`:  `merge(self="rust_field_name_in_this_struct", other="rust_field_name_on_other_struct")`. For composite fields use multiple `merge`.
ignore_wildcard | No selection for `**` and `*`| 
alias | Alias for `sql_join`  | 
table | Table name for joins and merges | 
role | Required role for field access | `role="admin", role= "superadmin"` For multiple roles use multiple `role`.
