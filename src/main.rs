


//extern crate toql;
use toql::sql_mapper::SqlMapper;
use toql::sql_mapper::FieldHandler;
use toql::sql_mapper::MapperOptions;
use toql::sql_mapper::BuildOptions;
use toql::user_query::UserDto;
use toql::query::Query;
use toql::query_parser::QueryParser;
use toql::query_parser::*;


fn main() {
    println!("Hello, world!");


     //let c=  parse_color();

    let query = QueryParser::parse("*, ((search MA \"Suche\"; +2 username EQ \"hallo\", age !GT 0)),.archive IN 0 1; id EQ 0");
    let query = QueryParser::parse("((age !EQ Hallo))");

   // let q = QueryParser::parse("id, name");

    //let u = UserDto::find_for_id(5);

    //let query = Query::new("id, name".to_string());
    //let query = QueryParser::parse("is, name");

    struct Test;
    impl FieldHandler for Test {

    }
    let t = Test{};
    
    let mut mapper = SqlMapper::new();
    mapper
        .map_field("id", "t1.id", MapperOptions::new().select_always(true).use_for_count_query(true))
        .map_field("username", "t1.username", MapperOptions::new())
        .map_field("archive", "t1.archive", MapperOptions::new())
        .map_field("age", "t1.age", MapperOptions::new())
        .map_handler("search", Box::new(t) , MapperOptions::new())
        ;
        
    let result = mapper.build(query, BuildOptions::new());
    println!("SELECT: {}", result.select_clause);
    println!("WHERE: {}", result.where_clause);
    println!("HAVING: {}", result.having_clause);
    println!("ORDER: {}", result.order_by_clause);
    println!("W PARAM: {:?}", result.where_params);
    println!("H PARAM: {:?}", result.having_params);
 
    
    //let s = mapper.build_sql("test");
   // let u2 = UserDto::find_for_toql(&mapper, &result.where_clause, 0, 10);
    
}

