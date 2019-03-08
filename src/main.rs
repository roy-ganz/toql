
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



impl toql :: sql_mapper :: Mappable for User { 
  fn map ( mapper : & mut toql :: sql_mapper :: SqlMapper , toql_path : & str , sql_alias : & str ) { 
      mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) {"" } else { "_" } , "id" ) , 
          & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "id" ) , 
          toql :: sql_mapper :: MapperOptions :: new ( ) ) ; 
          
    mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "username" ) , 
    & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "username" ) ,
     toql :: sql_mapper :: MapperOptions :: new ( ) .count_query(true) ) ; 
     } 
  }

 
 impl toql :: sql_mapper :: Mappable for Book { 
   fn map ( mapper : & mut toql :: sql_mapper :: SqlMapper , toql_path : & str , sql_alias : & str ) { 
	mapper . map_field_with_options ( 
		& format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "id" ) , 
		& format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "id" ) , 
	toql :: sql_mapper :: MapperOptions :: new ( ) ) ; 

	mapper . map_field_with_options ( 
	& format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "title" ) , 
	& format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "title" ) , 
	toql :: sql_mapper :: MapperOptions :: new ( ) ) ; 
	
	mapper . map_field_with_options ( 
		& format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "_author_id" ) , 
		& format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "author_id" ) , 
		toql :: sql_mapper :: MapperOptions :: new ( ) ) ; 

	mapper . map_join :: < User > ( "author" , "a" ) ; 
  } 
  
  }

    /* fn author_from_row_i(row: Vec, &mut i) ->User {
        User {
          id: row[i+= 1],
          username: row.take("author_username"),
          books: Vec::new(),
          other: String::new()
      }
    }
  fn book_from_row(row: Vec) ->Book {

      Book {
             id: row.take("id"),
             title: row.take("title"),
             author_id: row.take("_author_id"),
             author: Some(author_from_selection(row))                // ev if row.get("author_id").is_none() {None} else { Some()}
            }
      } */

  


  fn main() {
      println!("Hello, world!");

      let mut mu = toql::sql_mapper::SqlMapper::map::<Book>();
      mu.join("author", "LEFT JOIN User a on (author_id = a.ud)");

        // Should roles make field null or return err?

      let q= toql::query_parser::QueryParser::parse("id, title, author_id, author_username");

      let r = toql::sql_builder::SqlBuilder::new().build(&mu, &q.unwrap());

      assert_eq!( "SELECT id, title FROM Book LEFT JOIN User u ON author_id = u.id", r.unwrap().sql_for_table("Book"));

      


  }