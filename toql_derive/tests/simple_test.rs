
extern crate toql_derive;
use toql_derive::Toql;


#[derive(Debug,  Clone, Toql)]
#[toql(tables="PascalCase")]
struct Book {
    id: u8,
    title: Option<String>,
    author_id :u8,

    #[toql(join="author_id <= id", alias="a")]
    author : Option<User>

}

#[derive(Debug, Clone, Toql)]
#[toql(tables="PascalCase")]
struct User{
    id: u8,         // Always selected

    #[toql(column="username", count_filter )]
    username: Option<String>,

    #[toql(skip)]
    other: String,
    
    #[toql(merge="id <= author_id")]
    books: Vec<Book>
}



#[test]
fn attributes(){
   
      let mut mu = toql::sql_mapper::SqlMapper::map::<Book>("book");
      mu.join("author", "LEFT JOIN User a on (b.author_id = a.id)");

      let q= toql::query_parser::QueryParser::parse("id, title, author_id");
      let r = toql::sql_builder::SqlBuilder::new().build(&mu, &q.unwrap());
      assert_eq!( "SELECT id, title, author_id FROM User", r.unwrap().to_sql());



     

    //UserDto::merge_books(users, books);

    
 
}

   

    

