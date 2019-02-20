


//extern crate toql;
use toql::sql_mapper::SqlMapper;
use toql::sql_mapper::FieldHandler;
use toql::sql_mapper::MapperOptions;
use toql::sql_builder::SqlBuilder;
use toql::user_query::UserDto;
use toql::query::Query;
use toql::query_parser::QueryParser;
use toql::query_parser::*;


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
        .map_field("id", "id", MapperOptions::new().select_always(true).count_query(true))
        .map_field("title", "title", MapperOptions::new())
        .map_field("published", "published_at", MapperOptions::new())
        .map_field("author_id", "a.age", MapperOptions::new())
        .map_field("author_username", "a.username", MapperOptions::new())
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
 
    
    //let s = mapper.build_sql("test");
   // let u2 = UserDto::find_for_toql(&mapper, &result.where_clause, 0, 10);
    
}

