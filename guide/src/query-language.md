# The Query Language

The toql query language is a string that defines which fields should be selected from a database table.

Fields can be filtered and ordered, they are separated by comma or semicolon to express AND or OR concatenation.

Fields preceded by a path refer to a depended table.


#### Example 1:
    id, +name, age gt 18
 is translated into 

    SELECT id, name, age WHERE age > 18 ORDER BY name ASC
 
 #### Example 2:
    id , .age eq 12; .age eq 15
 is translated into
 
    SELECT id WHERE age = 12 OR age = 15
 
 

 
