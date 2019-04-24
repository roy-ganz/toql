# The toql derive
Using the toql derive is the recommended way to use Toql with your project. While the derive can setup almost everything, it is sometimes required to do some fine tuning manually on the Mapper

Toql provides a powerful derive that can
- Map struct fields to toql fields and table columns
- Create query builder functions
- Define relationships through joins and merges
- define table, column and toql field properties
- Create highlevel database functions to load, insert and update structs
- Create integration with Rocket

## Typical usecase

With this simple code

  #[derive(Toql)]
  struct User {
  	  id: u32,
	  name: String,
}

we can now do the following
let conn = ...
let cache = SqlMapperCache::new();
User::insert_new_mapper(&cache);  // function from derive
let q = Query::new();
q.and(User::all_fields()).and(User::fields.id().eq(5)); // builder fields from derive
let user = User::load_one_from_mysql(q, &conn, &cache, false); // function from derive

## Mapping names
Names between struct fields, database and toql fields are mapped by default in a predictable way:
1. Table names are UpperCamelCase
2. Column names are snake_case
3. Toql fields are lowerCamelCase (depended struct are separated by an underscore)
4. A field can map to an expression instead of a column.

To adjust the naming to an existing database use the toql attributes. General renaming can be "SnakeCase", ....
To ignore a field annotate it with skip, and optionally specify what to ignore 
#[toql(skip)] is the same as #[toql(skip="query,insert,update,delete")

#[derive(Toql)]
#[toql(tables="SnakeCase", columns="UpperCase")]
  struct User {
  	  id: u32
  	  #[toql(field="name2" column="name_2")
	  name: String,
  	  #[toql(skip)] 
	  age: u8
	  #[toql(sql="SELECT * FROM Phone p WHERE p.user= ..id")] // Two dots will be replaced with table alias, here user.id
	  numberOfPhones: u8
  }

## Optional fields
Each field in a toql query can individually be selected. However fields must be optional for this, otherwise they will always be selected in the SQL statement regardless of the query.

  struct User {
  	  id: u32,															// Always selected in SQL
	  name: Option<String>  								// Optional field
	  middlename: Option<Option<String>>  // Optional field of nullable column
	  #[toql(select)]
	  middlename: Option<String>  // Nullable column, always selected in SQL
  }

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