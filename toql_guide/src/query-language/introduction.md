# The Query Language

The Toql query language is a normal string that list all query fields, that should be retrieved from a database. 

Besides selection, query fields can also be filtered and ordered. 

They are separated either by comma or semicolon. If a filter is applied a comma will join the filters with AND, a semicolon with OR.

#### Example 1:
    id, +name, age gt 18
 is translated into 

    SELECT id, name, age WHERE age > 18 ORDER BY name ASC
 
#### Example 2:
    id, .age eq 12; .age eq 15
 is translated into
 
    SELECT id WHERE age = 12 OR age = 15


The actual SQL depends not only of the query but also of the mapper setup. 
 
 
 

 
