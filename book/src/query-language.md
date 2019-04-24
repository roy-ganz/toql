# The Query Language

The toql Query language is a string that names all the fields to select in a database table. Each field can be filtered and ordered. Fields are separated by comma or semicolon to join filter with  AND resp. OR.

#### Example 1:
    id, +name, age gt 18
 will  be translated into 

    SELECT id, name, age WHERE age > 18 ORDER BY name ASC
 
 #### Example 2:
    id , .age eq 12; .age eq 15
 will be translated into
 
    SELECT id WHERE age = 12 OR age = 15
 
 ## Selecting a field
 Here are some selection examples
 
 - Names without underscore, represent names in the table, the query is run, e.g id, name, fullName, emailAddress
 - Names with underscore are called fields with a path. They are mapped to a joined table or another table relationship. In this case a join will be added to the SQL statement or a second SQL query must be run to query and merge the table records. Fields with paths lokk like this:
 book_id, book_title, book_createdBy_id
 - To use a field only for filtering, but not for selection hide it with a dot, e.g .age, .book_id

#### Example 1
    id, book_id, .age eq 50
 
 will become (SQL Mapper must be told how to join)
 
    SELECT a.id, b.id FROM User a JOIN Book b ON (a.book_id = b.id) WHERE a.age > 50

## Ordering
Fields can be ordered in ascening or descending way. To order multiple fields use numers. If two fields have the same number the first field has higher importance

#### Example 1
    +id, +title
 
 will become
 
    (...) ORDER BY id ASC, title ASC
 
 #### Example 2
    -2id, -1title
 
 will become
 
    (...) ORDER BY title DESC, id DESC
 
 ## Filtering
 
 Fields can be filtered with standart operations or custom functions.
 Operations:

Toql| Operation | Filter Example | SQL
---|---|---|---
eq | equal	|	age eq 50  | age = 50 
eqn| equal null	|age eqn	|	age IS NULL  
ne	|not equal	|name ne 'foo'	|name <> 'foo'
nen |not equal null|	age nen|	age IS NOT NULL



