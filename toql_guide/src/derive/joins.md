
# Joins
A struct can refer to another struct. This is done with a SQL join. 

Joins are automatically added to the SQL statement in these situations:
-  Fields in the Toql query refer to another struct through a path: `user_phoneId`.
-  Fields on a joined struct are always selected: `#[toql(select_always)`. 
-  Fields on a joined struct are not `Option<>`: `id: u64`.

#### Example:

The Toql query `id` translates this

```rust
struct User {
	 id: u32,	
	 name: Option<String>
	 #[toql(sql_join(self="mobile_id" other="id"))]  
	 mobile_phone : Option<Phone>

	 #[toql(sql_join(self="country_id" other="id"))]  
	 country : Country
}

struct Country {
	id: String // Always selected
}

struct Phone {
	id : Option<u64>, 
}
```
into

```sql 
SELECT user.id, null, null, country.id FROM User user 
INNER JOIN Country country ON (user.country_id = country.id)
```

While the Toql query `id, mobilePhone_id` for the same structs translates into

```sql 
SELECT user.id, null, mobile_phone.id, country.id FROM User user 
LEFT JOIN Phone mobile_phone ON (user.mobile_id = mobile_phone.id)
INNER JOIN Country country ON (user.country_id = country.id)
```

## Naming and aliasing
The default table names can be changed with `table`, the alias with `alias`. 

The Toql query `id` for this struct

```rust
#[toql table="Users", alias="u"]
struct User {
	 id: u32,	
	 name: Option<String>
	 #[toql(sql_join(self="mobil_id", other="id"), table="Phones", alias="p")]  
	 mobile_phone : Option<Phone>
}
```

now translates into
```sql 
SELECT u.id, null, p.id FROM Users u LEFT JOIN Phones p ON (u.mobile_id = p.id)
```

## Join Attributes
SQL joins can be defined with
- *self*, the column on the referencing table. If omitted the struct field's name is taken.
- *other*, the column of the joined tabled.
- *on*, an additional SQL predicate. Must include the table alias.

For composite keys use multiple `sql_join` attributes.

#### Example
``` rust
 	#[toql(sql_join(self="country_id", other="id"), sql_join(self="language_id", other="language_id", on="country.language_id = 'en'") ]  
	country : Option<Country>
```


## Left joins and inner joins
There are four different join situations in a Rust struct. Depending on the situation a LEFT JOIN or INNER JOIN is added to the generated SQL.

Notice that inner joins are **always** added to the generated SQL, because inner joins filter the resulting dataset and this behaviour must usually be preserved.
(Table rows that have no valid inner join to another table row do not appear in the output result).

Left joins however are only added, if the query string refers to the related table. Left joins do not filter output rows.

### `Option<Option<T>>`  
A selectable optional relation. A LEFT JOIN is invoked if the query requests it.

Possible results are:
- `None`, Field is not selected by query or mapper.
-  `Some(None)`, Field is selected, but no related table exists (Foreign key is NULL).
-  `Some(Some(T))`, Field is selected and related table exists.

### `#[toql(preselect)] Option<T>>`  
An always selected optional relation. A LEFT JOIN is invoked if the query requests it.

Possible results are:
- `None`, The related table does not exist  (Foreign key is NULL).
- `Some(T)`, The value of the related table. 

### `Option<T>>`  
A selectable relation that must not be null. An INNER JOIN is always invoked.

Possible results are:
- `None`, Field is not selected by query or mapper.
- `Some(T)`, The value of the related table. 

### `T`  
An always selected relation, that must not be null. An INNER JOIN is always invoked.










