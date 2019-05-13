# Selecting fields

Fields are selected if they are mentioned in the query. 

- Names without underscore, represent typically columns or expressions from the table the query is run against. `id, name, fullName, emailAddress`

- Fields with underscores are called fields with a path. They are mapped to a joined table or another table relationship. For a join relationship, the join will be added to the SQL statement if the field is selected. For a merge relationship a second SQL query must be run to query and merge the dependency. `book_id, book_title, book_createdBy_id`

- To use a field only for filtering, but not for selection hide it with a dot. `.age, .book_id`

#### Example 1
    id, book_id, .age eq 50
 
 is translated into (SQL Mapper must be told how to join)
 
    SELECT a.id, b.id, null FROM User a JOIN Book b ON (a.book_id = b.id) WHERE a.age > 50

## Wildcards
There are two wildcards to select multiple fields. They can neither be filtered nor ordered.

- __**__ selects all mapped fields (top level and dependencies). Usefull for development.
- __*__ selects all fields from the top level

- __*path\_**__ selects all fields from the dependency _path_.

Fields can be excluded from the wildcard by setting them to `ignore wildcard`. 

Fields that require the user to have a special role are also ignored, if the query does not have that role.


### Example 1

`id, age, book_id, .age eq 50`

can be expressed as

`*, book_*, .age eq 50`

can be expressed as

`**, .age eq 50`
 
 and is translation into
 
`SELECT id, book_id, age FROM FROM User a JOIN Book b ON (a.book_id = b.id) WHERE a.age > 50`
 
_Note that the `age` field is selected with **_
 
