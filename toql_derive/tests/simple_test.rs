#[macro_use]
extern crate toql_derive;
use toql_derive::Toql;

use toql;





#[derive(Debug,  Clone, Toql)]
#[toql(tables="PascalCase")]
struct Book {
    id: u8,
    title: Option<String>,
    author_id :u8,

    #[toql(join="author_id <= id", alias="a")]
    author : Option<self::User>

}

#[derive(Debug, Clone, Toql)]
#[toql(tables="PascalCase")]
struct User{
    
    id :u8,         // This is always selected

    #[toql(column="username", count_query )]
    username: Option<String>,

    #[toql(skip )]
    other: String,

    
    #[toql(merge="id <= author_id")]
    books: Vec<self::Book>
}

/* 
impl toql :: mysql :: FromResultRow < Book > for Book { 
    fn from_row_with_index ( mut row : & mut mysql :: Row , i : & mut usize ) -> Result < Book , mysql :: error :: Error > { Ok ( Book { } ) } } 

impl Book { } 
impl toql :: sql_mapper :: Mappable for Book { 
    fn mapper ( table_alias : & str ) -> toql :: sql_mapper :: SqlMapper { 
        let m = toql :: sql_mapper :: SqlMapper :: new ( "Book" ) ; 
        Self::map ( & m , "" , table_alias ) ; m 
    }


fn map ( mapper : & mut toql :: sql_mapper :: SqlMapper , toql_path : & str , sql_alias : & str ) { 
    mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "id" ) ,
     & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "id" ) ,
     toql :: sql_mapper :: MapperOptions :: new ( ) .select_always(true) ) ; 
     mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "title" ) ,
      & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "title" ) , 
      toql :: sql_mapper :: MapperOptions :: new ( ) ) ;
       mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "authorId" ) , 
       & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "author_id" ) , 
       toql :: sql_mapper :: MapperOptions :: new ( ) .select_always(true) ) ;
        mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "author" ) ,
         & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "author" ) ,
          toql :: sql_mapper :: MapperOptions :: new ( ) ) ; } 
    }

impl toql :: mysql :: FromResultRow < User > for User { 
    fn from_row_with_index ( mut row : & mut mysql :: Row , i : & mut usize ) -> Result < User , mysql :: error :: Error > { 
        Ok ( User { id : row . take ( { * i } ) . unwrap ( ) , 
        full_name : row . take ( { *i +=1; * i } ) . unwrap ( ) , 
        modified_by_id : row . take ( { *i +=1; * i } ) . unwrap ( ) ,
         modified_by : Some ( < UserRef > :: from_row_with_index ( & mut row , { *i +=1; i } ) ? ) , 
         spoken_languages : Vec :: new ( ) , number_of_worksheets : row . take ( { *i +=1; * i } ) . unwrap ( ) , 
         rights : row . take ( { *i +=1; * i } ) . unwrap ( ) , 
         permissions : row . take ( { *i +=1; * i } ) . unwrap ( ) } ) } } 
         impl User { pub fn merge_spoken_languages ( t : & mut Vec < User > , o : Vec < UserLanguage > ) { 
             toql :: sql_builder :: merge ( t , o , | t | Option::from(t.id ) , | o | 
             Option::from(o. user_id) , | t , o | t . spoken_languages . push ( o ) ) ; } 
             
             pub fn fields ( ) -> UserFields { UserFields :: new ( ) } }
              pub struct UserFields ( String ) ; impl UserFields { pub fn new ( ) -> Self { 
                  Self :: from ( String :: from ( "" ) ) } 
                  pub fn from_path ( path : String ) -> Self { Self ( path ) } pub const ID : & \'static str = "id" ; 
                  pub const FULL_NAME : & \'static str = "fullName" ; pub const MODIFIED_BY_ID : & \'static str = "modifiedById" ; 
                  pub const NUMBER_OF_WORKSHEETS : & \'static str = "numberOfWorksheets" ; pub const RIGHTS : & \'static str = "rights" ; 
                  pub const PERMISSIONS : & \'static str = "permissions" ; pub fn id ( mut self ) -> toql :: query :: Field { self . 0 . push_str ( "id" ) ; 
                  toql :: query :: Field :: from ( self . 0 ) } 
                  pub fn full_name ( mut self ) -> toql :: query :: Field { self . 0 . push_str ( "full_name" ) ; 
                  toql :: query :: Field :: from ( self . 0 ) } 
                  pub fn modified_by_id ( mut self ) -> toql :: query :: Field { self . 0 . push_str ( "modified_by_id" ) ; 
                  toql :: query :: Field :: from ( self . 0 ) } 
                  pub fn number_of_worksheets ( mut self ) -> toql :: query :: Field { self . 0 . push_str ( "number_of_worksheets" ) ; 
                  toql :: query :: Field :: from ( self . 0 ) } 
                  pub fn rights ( mut self ) -> toql :: query :: Field { self . 0 . push_str ( "rights" ) ; toql :: query :: Field :: from ( self . 0 ) } 
                  pub fn permissions ( mut self ) -> toql :: query :: Field { self . 0 . push_str ( "permissions" ) ;
                   toql :: query :: Field :: from ( self . 0 ) } 
                   pub fn modified_by ( mut self ) -> UserRefPath { self . 0 . push_str ( "modifiedBy_" ) ; UserRefPath :: from ( self . 0 ) } } 
                   impl toql :: sql_mapper :: Mappable for User { fn mapper ( table_alias : & str ) -> toql :: sql_mapper :: SqlMapper 
                   { let mut m = toql :: sql_mapper :: SqlMapper :: new ( "User" ) ; 
                   Self :: map ( & mut m , "" , table_alias ) ; m } fn map ( mapper : & mut toql :: sql_mapper :: SqlMapper , toql_path : & str , sql_alias : & str ) { mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "id" ) , & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "id" ) , toql :: sql_mapper :: MapperOptions :: new ( ) .select_always(true) ) ; mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "fullName" ) , & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "full_name" ) , toql :: sql_mapper :: MapperOptions :: new ( ) ) ; mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "modifiedById" ) , & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "modified_by" ) , toql :: sql_mapper :: MapperOptions :: new ( ) ) ; mapper . map_join :: < UserRef > ( "user" , "modified_by" ) ; mapper . join ( "user" , &format!("LEFT JOIN User modified_by ON ( {}.modified_by  = modified_by. id)", sql_alias) ) ; mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "numberOfWorksheets" ) , & "(SELECT count(*) FROM Worksheet w WHERE w.owner_id = ..id)" . replace ( ".." , & format ! ( "{}." , sql_alias ) ) , toql :: sql_mapper :: MapperOptions :: new ( ) .select_always(true) ) ; mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "rights" ) , & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "rights" ) , toql :: sql_mapper :: MapperOptions :: new ( ) .ignore_wildcard(true) ) ; mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "permissions" ) , & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "permissions" ) , toql :: sql_mapper :: MapperOptions :: new ( ) ) ; } }")
 */

/* 
//# [ cfg ( feature = "mysqldb" ) ] 
impl toql :: mysql :: FromResultRow < Book > for Book { 
    fn from_row_with_index ( mut row : mysql :: Row , i : & mut usize ) -> Result < Book , mysql :: error :: Error > { 
        Ok ( Book { 
            id : row . take ( { i } ) , 
            title : row . take ( { *i +=1; i } ) , 
            author_id : row . take ( { *i +=1; i } ) , 
            author : Some ( < User > :: from_row_with_index ( row , { *i +=1; i } )  ), 
            } ) 
    } 
    } 
            
    impl Book { } 
    
    impl toql :: sql_mapper :: Mappable for Book { 
        fn map ( mapper : & mut toql :: sql_mapper :: SqlMapper , toql_path : & str , sql_alias : & str ) { 
            mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "id" ) , 
            & format ! ( "{}{}{}" , sql_alias , 
            if sql_alias . is_empty ( ) { "" } else { "." } , "id" ) , 
            toql :: sql_mapper :: MapperOptions :: new ( ) ) ; 
            
            mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "title" ) , 
            & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "title" ) , 
            toql :: sql_mapper :: MapperOptions :: new ( ) ) ; 
            
            mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "author_id" ) , 
            & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "author_id" ) , 
            toql :: sql_mapper :: MapperOptions :: new ( ) ) ; 
            
            mapper . map_join :: < User > ( "author" , "a" ) ; 
            } 
    }


//# [ cfg ( feature = "mysqldb" ) ] 
impl toql :: mysql :: FromResultRow < User > for User { 
    fn from_row_with_index ( mut row : mysql :: Row , i : & mut usize ) -> Result < User , mysql :: error :: Error > { 
        Ok ( User { 
            id : row . take ( { i } ) , 
            username : row . take ( { *i +=1; i } )  } ) 
    } 
} 
            
impl User { 
    pub fn merge_books ( t : & mut Vec < User > , o : Vec < Book > ) { 
        toql :: sql_builder :: merge ( t , o , | t | Option::from(t. id) , | o | Option::from(o.author_id ) , | t , o | t . books . push ( o ) ) ; 
    } 
} 
impl toql :: sql_mapper :: Mappable for User { 
    fn map ( mapper : & mut toql :: sql_mapper :: SqlMapper , toql_path : & str , sql_alias : & str ) { 
        mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "id" ) , 
        & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "id" ) , 
        toql :: sql_mapper :: MapperOptions :: new ( ) ) ; 
        
        mapper . map_field_with_options ( & format ! ( "{}{}{}" , toql_path , if toql_path . is_empty ( ) { "" } else { "_" } , "username" ) ,
         & format ! ( "{}{}{}" , sql_alias , if sql_alias . is_empty ( ) { "" } else { "." } , "username" ) , 
         toql :: sql_mapper :: MapperOptions :: new ( ) .count_query(true) ) ; 
    } 
}
 */

// New 



#[test]
fn test_simple(){

        
   
      println!("Hello, world!");

      let mut mu = toql::sql_mapper::SqlMapper::map::<Book>();
      mu.join("author", "LEFT JOIN User a on (b.author_id = a.id)");

      let q= toql::query_parser::QueryParser::parse("id, title, author_id");

      let r = toql::sql_builder::SqlBuilder::new().build(&mu, &q.unwrap());

      assert_eq!( "SELECT id, title, author_id FROM User", r.unwrap().sql());



     

    //UserDto::merge_books(users, books);

    
 
}

   

    

