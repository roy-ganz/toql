
# Joins
A struct can refer to another struct. This is done with a SQL join. 

Joins are automatically added to the SQL statement in these situations:
-  Fields in the Toql query refer to another struct through a path: `user_phoneId`.
-  Fields on a joined struct are always selected: `#[toql(preselect)`. 
-  Fields on a joined struct are not `Option<>`: `id: u64`.
-  A related struct is an inner join.

#### Example:

The Toql query `id` translates this

```rust
struct User {
	 id: u32,	
	 name: Option<String>
	 #[toql(sql_join(self="mobile_id" other="id"))]  
	 mobile_phone : Option<Phone> // Selectable inner join

	 #[toql(sql_join(self="country_id" other="id"))]  
	 country : Country // Inner Join
}

struct Country {
	id: String // Always selected
}

struct Phone {
	id : Option<u64>, // Can be null
}
```
into

```sql 
SELECT user.id, -snip-, country.id FROM User user 
INNER JOIN Country country ON (user.country_id = country.id)
```

While the Toql query `id, mobilePhone_id` for the same structs translates into

```sql 
SELECT user.id, -snip-, mobile_phone.id, country.id FROM User user 
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
	 #[toql(preselect, sql_join(self="mobil_id", other="id"), table="Phones", alias="p")]  
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
```rust
 	#[toql(preselect, sql_join(self="country_id", other="id"), sql_join(self="language_id", other="language_id", on="country.language_id = 'en'") ]  
	country : Option<Country>
```


## Left joins and inner joins
There are four different join situations in a Rust struct. Depending on the situation a LEFT JOIN or INNER JOIN is added to the generated SQL.

Notice that inner joins are **always** added to the generated SQL, because inner joins filter the resulting dataset and this behaviour must usually be preserved.
(Table rows that have no valid inner join to another table row do not appear in the output result).

Left joins however are only added, if the query string refers to the related table. Left joins cannot  reduce output rows.

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


## Sidenote for SQL generation
Toql can select individual fields. This is done for simple columns by selecting `null` instead of the table column and checking if the resulting column type is accordingly. 

For left joins this is not possible because the database uses the same technique to indicate missing left joins. Therefore Toql generates for left joins
a extra discriminator expression to distinguish a missing left join from an unselected left join.

If you watch the generated SQL output, you will notice that JOIN statements look slightly more complicated from Toql than from some other frameworks (hello eclipse link). 

This is because Toql builds correctly nested JOIN statements that reflect the dependencies among the joined structs. Any SQL builder that simply concatenates inner joins and left joins may accidentally turn left joins into inner joins. This database behaviour is not well known and usually surprises users - Toql avoids this.








