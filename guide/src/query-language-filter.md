
# Filtering
 
Fields can be filtered by adding a filter to the field name. 

- Filters are case insensitiv.
- Arguments are separated by whitespace.
- Strings and enum arguments are enclosed with single quotes.
- Boolean arguments are expressed with numbers 0 and 1.


## Filter operations

Toql| Operation | Example | SQL _MySQL_
---|---|---|---
eq | _equal_	|	age eq 50  | age = 50 
eqn| _equal null_	|age eqn	|	age IS NULL  
ne	| _not equal_	|name ne 'foo'	|name <> 'foo'
nen | _not equal null_|	age nen|	age IS NOT NULL
gt | _greater than_ | age gt 16 | age > 16
ge | _greater than or equal_ | age ge 16 | age >= 16
lt | _less than_ | age lt 16 | age < 16
le | _less than or equal_ | age le 16 | age <= 16
bw | _between_ | age bw 16 20 | age BETWEEN 16 AND 20
in | _includes_ | name in 'Peter' 'Susan' | name in ('Peter, 'Susan')
out | _excludes_ | age out 1 2 3 | name not in (1, 2, 3)
re | _matches regular expression_ | name re ".\*" | name REGEXP '.*'
sc | _set contains_ | role sc 'ADMIN' 'SUPERADMIN' | FIND_IN_SET(role, 'ADMIN,SUPERADMIN') > 0
fn | _custom function_ | search fn ma 'arg1' | _depends on implementation_



## Custom functions
Custom functions are applied through the `FN` filter. They must be registered on the [SQL Mapper](sql-mapper.md).

