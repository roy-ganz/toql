# The Query Language

The Toql query language is a normal string that list all query fields, that should be retrieved from a database. 

Besides selection, query fields can also be filtered and ordered. 

They are separated either by comma or semicolon. If a filter is applied a comma will join the filters with AND, a semicolon with OR.

#### Example 1:
    id, +name, age gt 18
 is translated into 

    SELECT t0.id, t0.name, t0.age FROM ... WHERE t0.age > 18 ORDER BY t0.name ASC
 
#### Example 2:
    id, .age eq 12; .age eq 15
 is translated into
 
    SELECT t0.id From ... WHERE t0.age = 12 OR t0.age = 15


 
 
 

 
