# Selections

A typical query selects a lot of fields. Instead of writing out a long list of fields, predefined field lists can be [mapped](../3-api/9-selections.md)

The list can then be selected with a `$` and the selection name.

```toql 
$mySelection, $otherSelection
```

There is a set of predefined selections:

|Selection | Scope|
|----------|------|
| $std     | Standart selection, must be mapped|
| $        | Alias for $std |
| $cnt     | Fields that are considered for a count query, defaults to keys and preselects|
| $all     | All fields on a struct, including dependencies|
| $mut     | All mutable fields on a struct|

Selections can also refer to a dependency, using a path. 

To load the standart selection from a dependency `address` use

```toql 
$address_std
```

### Restriction on selection names
Selection names with 3 or less characters are reserved for internal purposes. 
User defined selection names must contain at least 4 characters.

