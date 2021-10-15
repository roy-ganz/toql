# Predicates

Normal query filters are applied to fields. 
Predicates overcome this limitation and can filter on any raw SQL predicate.

The behaviour of predicates must be [mapped](../4-derive/10-predicates.md), 
then they can be called with a `@`, the predicate name and zero or more arguments.

```toql 
@search 'peter', @updated, @tags 'island' 'fun'
```


Predicates can refer to a dependency, using a path. 

To search a dependency `address` use

```toql 
@address_search 'peter'
```


