


//extern crate toql;
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
   title: String,
   author_id: Option<u8>,
   author: Option<Author>
 }

#[derive(Debug, Clone)]
 struct Author {
   id: u8,
   title: String,
   books: Vec<Book>
 }

fn main() {
    println!("Hello, world!");


     //let c=  parse_color();

    //let query = QueryParser::parse("*, ((search MA \"Suche\"; +2 username EQ \"hallo\", age !GT 0)),.archive IN 0 1; id EQ 0");
    //let query = QueryParser::parse("title EQ \"Hallo\"");
     //let query = QueryParser::parse("book_title, book_author_username EQ 'fritz'");
     //  let query = QueryParser::parse("fooId, bar_id, author_id, author_username, author_book_id EQ 5");
  let query = QueryParser::parse("id, ((title EQ 'Foo'; (title !EQ 'bar'))), id NE 3").unwrap();
        
   // let query = QueryParser::parse("*");

   // let q = QueryParser::parse("id, name");

    //let u = UserDto::find_for_id(5);

    //let query = Query::new("id, name".to_string());
    //let query = QueryParser::parse("is, name");

    struct Test;
    impl FieldHandler for Test {

    }
   
    
    let mut mapper = SqlMapper::new();
     mapper
        .join("author", "LEFT JOIN author a ON (id = a.book_id)")
        .map_field_with_options("id", "id", MapperOptions::new().select_always(true).count_query(true))
        .map_field("title", "title")
        .map_field("published", "published_at")
        .map_field("author_id", "a.age")
        .map_field("author_username", "a.username")
        ; 
       
     
    //let result = mapper.build(query, BuildOptions::new());


    let result = SqlBuilder::new()
       .build(&mapper, &query).unwrap();
           
    //  assert_eq!("SELECT id, username, b.id FROM User JOIN Book b ON (id = b.id) WHERE b.id = ?", result.sql_for_table("User"));

  /*  let result= builder.for_role("hkhk")
    // .with_restriction( QueryParser::parse("id neq \"hfkjsh\""))
     .with_join("user")
     .alias("t0")
     .build(&mapper, &query); */
     //.build_subpath("fdjdlkf", "hkjhkj")

    println!("Sql is: {}", result.sql_for_table("User"));

    println!("SELECT: {}", result.select_clause);
    println!("WHERE: {}", result.where_clause);
    println!("HAVING: {}", result.having_clause);
    println!("ORDER: {}", result.order_by_clause);
    println!("W PARAM: {:?}", result.where_params);
    println!("H PARAM: {:?}", result.having_params);
 

    // Test merging
  let mut books = vec![ Book{id:5, title:String::from("Titel 5"), author_id:Some(1), author: None}, 
              Book {id:2, title: String::from("Title 2"), author_id:Some(1), author: None}, 
              Book {id:1, title: String::from("Title 1"), author_id: Some(2), author: None} ];

  let mut authors = vec![ Author{id:1, title:String::from("Author A"), books: Vec::new()},
                        Author{id:2, title:String::from("Author B"), books: Vec::new()}, 
   ];

  // merge! (authors => books.author, author.id =>  book.author_id);
  //dmerge_authors(&mut books, authors);

  // merge! (books => author.books, book.author_id => author.id);

struct User{
    
    id : u8,         // This is always selected
    username: Option<String>,
    books: Vec<Book>
}

impl User { 
  pub fn merge_books ( t : & mut Vec < User > , o : Vec < Book > ) { 
      toql :: sql_builder :: merge ( t , o , | t | Option::from(t. id) , | o | Option::from(o.author_id ) , | t , o | t . books . push ( o ) ) ; 
      } 
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
		& format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "author_id" ) , 
		& format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "author_id" ) , 
		toql :: sql_mapper :: MapperOptions :: new ( ) ) ; 

	mapper . map_join :: < User > ( "author" , "a" ) ; 
  } 
  
  }
  // works
  
  /*
  Struct Book {
    id: u8,
    author_id : u8,

    #[toql( merge "id => author_id") ]
    author: Option<Author>,
  }
*/
  //genmerge_optional(&mut books, authors, |b, a| b.author = Some(a), |b| b.author_id, |a| a.id);

  
/*
  Struct Author {
    id: u8,

    #[toql( merge "author_id => id") ]
    books: Vec<Book>,

  }
*/
  merge(&mut authors, books, |a, b| a.books.push(b), |a| Option::from(a.id), |b| Option::from(b.author_id));



  println!("{:?}", authors);

  let b1 = Book {id:6, title: String::from("hkjh"), author_id: Some(5), author: None};
  let mut test = HashMap::new();
  test.insert((b1.id,b1.title), "test");

  println!("{:?}", test);



    //let s = mapper.build_sql("test");
   // let u2 = UserDto::find_for_toql(&mapper, &result.where_clause, 0, 10);
    
}

 // Single merge
 /*  fn merge_authors(books : &mut Vec<Book>, authors: Vec<Author> ) {
    
    // Build index to lookup books by author id
    let mut index : HashMap<u8, usize> = HashMap::new();

    for (i,a) in authors.iter().enumerate() {
      index.insert(a.id, i);
    }

    for b in books {
      let author_index = index.get(&b.author_id).unwrap();
      let author = authors.get(*author_index).unwrap();

      // Author must be copied, because same author can be in multiple books
      b.author = Some(author.clone());
    } 

  } */

  // Draining merge
  // Variables a.id, author_.id
  // Type of a.id
  // Field 
  //maybe macro merge! (books.author => authors, book.author_id => author.id );

 
/* 
#[derive(Debug, Clone)]
 struct Book {
   id: u8,
   title: String,
   author_id: u8,

   #toql(merge="auther_id ON id")
   author: Option<Author>
 }
  */

 fn merge<T,  O,   K, F, X, Y>( this: &mut std::vec::Vec<T>, mut other: Vec<O>, assign: F, tkey : X, okey: Y )
 where O:Clone,
      K: Eq + std::hash::Hash,
      F: Fn(&mut T, O) , 
      X: Fn(&T) -> Option<K>, 
      Y: Fn(&O) -> Option<K>
  {

    // Build index to lookup all books of an author by author id
    let mut index : HashMap<K, Vec<usize>> = HashMap::new();

    for (i,b) in this.iter().enumerate() {
      match  tkey(&b) {
         Some(k) => {
            let v = index.entry(k).or_insert(Vec::new());
             v.push(i);
         },
         None => {}
      }
     
     // let v = index.entry(b.author_id).or_insert(Vec::new());
     
    }

    // Consume all authors and distribute 
    for a in other.drain(..) {

      // Get all books for author id
      //let vbi = index.get(a.id).unwrap();
      match &okey(&a) {
        Some(ok) => {
          let vbi = index.get(ok).unwrap();

          // Clone author for second to last books
          for bi in vbi.iter().skip(1) {
            //let  mut  b = this.get_mut(*bi).unwrap();
            if let  Some(mut  b) = this.get_mut(*bi) {
              assign(&mut b, a.clone());
            }
           
          }

          // Assign drained author for first book
          let fbi= vbi.get(0).unwrap();
          //let mut b = this.get_mut(*fbi).unwrap();
          if let Some(mut b) = this.get_mut(*fbi) {
            assign(&mut b, a.clone());
          }
        }
        None => {}

      }
     
    } 

 }

/* 
  fn dmerge_authors(books : &mut Vec<Book>, mut authors: Vec<Author> ) {
    
    // Build index to lookup all books of an author by author id
    let mut index : HashMap<u8, Vec<usize>> = HashMap::new();

    for (i,b) in books.iter().enumerate() {
      let v = index.entry(b.author_id).or_insert(Vec::new());
      v.push(i);
    }

    // Consume all authors
    for a in authors.drain(..) {

      // Get all books for author id
      let vbi = index.get(&a.id).unwrap();
      
      // Clone author for second to last books
      for bi in vbi.iter().skip(1) {
        let b = books.get_mut(*bi).unwrap();
        b.author = Some(a.clone());
      }

      // Assign drained author for first book
      let fbi= vbi.get(0).unwrap();
      let b = books.get_mut(*fbi).unwrap();
      b.author = Some(a);
    } 

  }

  fn dmerge_books(authors : &mut Vec<Author>, mut books: Vec<Book> ) {
    
    // Build index to lookup all authors of a book by id
    let mut index : HashMap<u8, Vec<usize>> = HashMap::new();

    for (i,a) in authors.iter().enumerate() {
      let v = index.entry(a.id).or_insert(Vec::new());
      v.push(i);
    }

    // Consume all books
    for b in books.drain(..) {

      // Get all authors for book by author_id
      let vbi = index.get(&b.author_id).unwrap();
      
      // Clone book for second to last books
      for bi in vbi.iter().skip(1) {
        let a = authors.get_mut(*bi).unwrap();
        a.books.push(b.clone());
      }

      // Assign drained book for first author
      let fbi= vbi.get(0).unwrap();
      let a = authors.get_mut(*fbi).unwrap();
      a.books.push(b);
    } 

  } */

