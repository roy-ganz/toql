
use std::collections::HashMap;
use toql::sql_mapper::SqlMapper;
use toql::sql_mapper::FieldHandler;
use toql::sql_mapper::MapperOptions;
use toql::sql_builder::SqlBuilder;
use toql::user_query::UserDto;
use toql::query::Query;
use toql::query_parser::QueryParser;
use toql::query_parser::*;

#[derive(Debug, Clone)]
struct Book {
    id: u8,
    title: Option<String>,
    author_id :u8,
  
    author : Option<User>

}

#[derive(Debug, Clone)]
struct User{
    id :u8,         // This is always selected
    username: Option<String>,
   other: String, // skipped
    books: Vec<Book>
}



  use toql::query::Field;

/* struct Fish ();
struct Market ();

struct Path<J, K>(String);
struct PathEnd<J>(String);

impl<J,K> Path<J,K> {
     fn join(mut self, path: Path<K,_>) -> Path {
        self.0.push('_'); 
        self.0.push_str(path);
        self
    }
    fn field(mut self, field: PathEnd<K> ) -> Field {
        self.0.push('_'); 
        self.0.push_str(field.0);
        Field::from(self.0)
    }
}



 let s :Path<Fish, Market>= Path("user");
 let d = s.field(PathField<Market>("id")); // -> Field */




  fn main() {

    let mut q = Query::new();
        q.and ("foo").and ("bar").or ("foo");
        assert_eq!("(foo,bar);foo", q.to_string());   

     // q.and( vec![Field::from("hkj").eq(5), Field::from("gh")]);
        
    //  q1.unwrap().prepend(q);

        

       //let q = Query::from(vec[Field::from("id"), Field::] ) 


   //   let mut mu = toql::sql_mapper::SqlMapper::map::<Book>("b");
    //  mu.join("author", "LEFT JOIN User a on (author_id = a.ud)");

        // Should roles make field null or return err?

    //  let q= toql::query_parser::QueryParser::parse("id, title, author_id, author_username");

    //  let r = toql::sql_builder::SqlBuilder::new().build(&mu, &q.unwrap());

    //  assert_eq!( "SELECT b.id, b.title FROM Book LEFT JOIN User u ON b.author_id = u.id", r.unwrap().sql_for_table("Book b"));

      


  }