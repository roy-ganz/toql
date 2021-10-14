# Toql Derive Reference

The derive provides struct level attributes and field level attributes. Here a list of all available attributes:

## Attributes for structs

Attribute | Description                             | Example / Remark
---- |---| ---|
tables  |   Table renaming scheme for struct and joins |  `CamelCase`, `snake_case`, `SHOUTY_SNAKE_CASE` or `mixedCase`
columns        | Column renaming scheme |
table | Table name for a struct or join | `table ="User"` on struct `NewUser` will access table `User`
skip_load | No code for load  | 
skip_mut |No code for insert, delete and update |
predicate |  Define a predicate | `predicate(name="test", sql="MATCH(..name, ..address) AGAINST (?)")` 
selection |  Define a selection | `selection(name="test", fields="*, address_street")` 
alias |Ignore calculated alias and use this alias instead| `alias="tb1"` 
auto_key | Key is generated in database | `auto_key=true` Updates struct keys after inserts.
roles  |  role restriction for load, update, insert, delete | `roles(update="admin;teacher", insert="admin")`

## Attributes for fields  

Attribute | Description | Example / Remark
---- |---| ---|
key| Primary key  |  For composite keys use multiple times. Skipped for insert, update.
column | column name | Use to overide default naming `column="UserNamE"`
sql | Map field to SQL expression | `sql="..title"` or `sql="(SELECT o.name FROM Other o WHERE o.id = ..other_id)"`, skipped for insert, update.
skip | Completly ignore field
skip_load | Ignore for loading
skip_mut | Ignore for updating
skip_wildcard | Don't include this field in wildcard selection | Use for expensive subselects
join | Required for fields that join other structs  | `join(columns(self="address_id", other="id"))`
merge |Required for fields that are Vec<>  | `merge(columns(self="address_id", other="id"))`
handler | Build SQL expression with code | `handler="get_handler"`, function returns struct implementing `toql::prelude::FieldHandler` 
aux_param| set aux_param | Use to give parameters to a field handler `aux_param(name="entity", value="USER")`
roles |  role restriction for load, update |   `roles(load="admin")`
foreign_key | The field is a foreign key. | Update that field too, if struct is joined. Rarely needed.











