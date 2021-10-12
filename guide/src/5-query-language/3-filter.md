
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
gt | _greater than_ | age gt 16 | age > 16com
ge | _greater than or equal_ | age ge 16 | age >= 16
lt | _less than_ | age lt 16 | age < 16
le | _less than or equal_ | age le 16 | age <= 16
bw | _between_ | age bw 16 20 | age BETWEEN 16 AND 20
in | _includes_ | name in 'Peter' 'Susan' | name in ('Peter, 'Susan')
out | _excludes_ | age out 1 2 3 | name not in (1, 2, 3)
re | _matches regular expression_ | name re ".\*" | name REGEXP '.*'
fn | _custom function_ | search fn ma 'arg1' | _depends on implementation_



## Custom functions
Custom functions are applied through the `FN` filter. They must be handled by a Field Handler. See API for details.


## Joining filters
A field can be filtered multiple times by adding multiple the filter expressions in the query.

To build complex filter expressions join filters by comma to express logical AND or semicolon for logical OR. 
Keep in mind that logical AND has higher precendence than logical OR. 

Use parens if required:

```toql 
age eq 12, animal eq 'chicken'; animal eq 'cow
```

is the same as

```toql 
(age eq 12, animal eq 'chicken'); animal eq 'cow
```

but different than

```toql 
age eq 12, (animal eq 'chicken'; animal eq 'cow)
```

Use the dot notation if you only want to filter a field without selecting it:

```toql 
age eq 12, .animal eq 'chicken'; .animal eq 'cow'
``` 

## Argument types
Toql onyl knows integers, floats and strings. Use the following table to express more types:

Type| Toql Example |Remark|
---|---|---|
bool| admin eq 1| 0, 1|
integer| limit  bw -12 5| |
float | price le 0.5e2| |
string| name in 'peter'| With ticks|
date |subscribeUntil le '2050-12-31'|SQL format|
time |start ge '08:30:00'| SQL format|
date time |finishedAt ge '2005-12-31 08:30:00'| SQL format|
