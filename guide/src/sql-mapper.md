#SQL Mapper

The sql mapper translates toql fields into sql column or expressions. The recommended workflow is to define a struct and use the 
toql derive to create all mapping functions.

However sometimes more customising must be done than the derive attributes allow. 
Especially
  - Add custom query functions ( _title FN CUSTOM "as$"_ )
  - Add fields that do not exist in struct ( _superseach eq "hallo"_ )
  - Map sql expressions programatically (sql permissions for authenticated users) 
  - Changing joins
  
 ## Typical usecase
  
  ```rust
  let cache = TableMapperRegistry::new();
  
  let mut user_mapper = User::insert_new_mapper(&cache);
  
  lt permission_sql = if true { "u."}
  
  user_mapper.add_field("permission", permission);
  ```
  
  ## Custom field handler 
  
  Using a custom field handler gives you most freedom. Here an example to match against multiple columns in MySQL.
  Note that in the database an index must be created for this to work. 
  
  struct MaHandler;
  impl FieldHandler for MaHandler {
  
  
  }
 
 user_mapper.add_field("search", MaHandler);  
 
 
 ## Overriding joins
 
 To change existing join or add a new one:
 user_mapper.alter_join("friend" "INNER JOIN sdds");
 
 