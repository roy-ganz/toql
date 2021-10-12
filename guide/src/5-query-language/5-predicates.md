# Ordering fields

Fields can be ordered in ascending `+` or descending `-` way.


#### Example 
`+id, -title`
 
is translated into
 
`--snip-- ORDER BY id ASC, title DESC`


## Ordering priority
Use numbers to express ordering priority. 
- Lower numbers have higher priority. 
- If two fields have the same number the first field in the query has more importance.
 
 #### Example 
`-2id, -1title, -2age`
 
 is translated into

 `--snip-- ORDER BY title DESC, id DESC, age DESC`
