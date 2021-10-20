
# Predicates
All normal filtering is based on fields, [see here](../5-query-language/4-filter.md). 
However sometimes you may have a completely different filter criteria that can't be mapped on fields. 

An example is the MySQL full text search. Let's do it:

```rust
#[derive(Toql)]
#[toql(predicate(name="search", 
		sql="MATCH (..firstname, ..lastname) AGAINST (?  IN BOOLEAN MODE)"))]

#[toql(predicate(name="street", 
		sql="EXISTS( SELECT 1 FROM User u JOIN Address a ON (u.address_id = a.id) \
		 	WHERE a.street = ? AND u.id = ..id)"))]
struct User {

 #[toql(key)]
 id: u64,

 firstname: String,
 lastname: String,
}
```

With the two predicates above you can seach for users that have a certain name with `@search 'peter'` 
and retrieve all users from a certain street with `@street 'elmstreet'`.

The question marks in the predicate are replaced by the arguments provided. 
If there is only one argument, it can also be used to build an `ON` predicate in a join. See [on aux param](7-joins.md).


## Reference

The full predicate syntax is
`predicate(name="..", sql="..", handler="..", on_aux_param="..", count_filter=true|false)` 
where 
- __name__ is the name of the predicate. It can be called in a Toql query with `@name ..`. 
  If a predicate is defined on a joined struct, that predicate can be called with a path
  `@path_name ..`. See [predicates in the query](5-query-language/5-predictes.md) for more details.
- __sql__ is a raw QL expression. Use `?` to insert a predicate param in the SQL, 
  `..` for the table alias and `<aux_param>` for an aux param value.
- __handler__ allows a custom predicate handler (build SQL with a function). 
  Provide a function name without parenthesis that return a struct that implement `toql::prelude::PredicateHandler`
- __on_aux_param__ sets the value of an aux_param. This aux param is only available when building custom joins
  and can only be used when the predicate takes exactly one argument. See [example](7-join.md).
- __count_filter__ determines if a predicate used in Toql query should also be used for [count queries](3-api/2-load.md). 
  Default is `false`.