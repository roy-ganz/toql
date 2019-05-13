# The toql derive
The recommended way to use Toql in your project is to use the Toql derive.

The derive build a lot of code.

This includes
- Mapping of struct fields to Toql fields and database
- Creating query builder functions
- Handling relationships through joins and merges
- Creating high level functions to load, insert, update and delete structs


## Typical usecase

With this simple code

 ```rust
	#[derive(Toql)]
	struct User {
		id: u32,
		name: String,
}
```

we can now do the following

```rust
use toql::mysql::load_one; // Load function from derive
use toql::mysql::update_one; // Update function from derive

let conn = --snip--
let cache = SqlMapperCache::new();
SqlMapper::insert_new_mapper::<User>(&mut cache); // Mapper function from derive
let q = Query::wildcard().and(User::fields.id().eq(5)); // Builder fields from derive
let user = load_one<User>(&q, &cache, &mut conn); 

user.age = Some(16);
update_one(&user); 
```



## Joins
A struct can refer to another struct using a join. If a field in the query mentions a field from that other struct a join clause will be added to the SQL statement. So the toql query "id, name, phone_number" will add a join clause for the following relationship:

struct User {
	 id: u32,														
	 name: Option<String>
	 #[toql(join="id <= user_id")]  // self.field <= other.field
	 mobile_phone : Phone
}

struct Phone {
	number: Option<String>,	// Non optional fields will always add join
	user_id : Option<u32>
}

The toql derive can only create inner joins (<=) or left joins (<-) for simple struct fields. For different column names in the referred struct, composite keys or join conditions use the sql mapper. In that case use an empty join attribute #toql(join) and alter the join in your program:

let cache = toql::SqlMapperCacher();
let mut mapper=	User::insert_new_mapper(&mut cache);
mapper.alter_join("phone", "LEFT JOIN user.id = phone.user_id AND phone.mobile = true");  // Ensure table aliases  are correct 

## Merge
A struct can also contain a collection of another struct. Since this cannot be done directly in SQL. Toql will execute multiple queries and merge the results. 

struct User {
	 id: u32,														
	 name: Option<String>
	 #[toql(merge="id = user_id")]  // rust comparison
	 mobile_phones : Vec<Phone>
}

struct Phone {
	number: Option<String>
	user_id : Option<u32>
}

# Insert, update and delete
Structs for toql queries include typically a lot of option fields. The toql derive creates insert, update and delete functions if reqested with the alter atribute. For composite keys mark multiple columns with the alterkey attribute and use a tuple in that order.

#[derive(Toq, Defaultl)]
#[toql(alter)]
struct User {
	#[toql(alterkey)]
	 id: u32,														
	 name: Option<String>
}
let u = User::default();
let x = User::insert(u); // returns key
User::update(5, u);
User::delete(5);

Notice that join and merge fields are excluded. To skip fields from insert or update functions, use the skip annotation. 
E.g. #[toql(skip="insert, delete")
