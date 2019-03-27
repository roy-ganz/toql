extern crate toql;

    use toql::query_parser::QueryParser;
    use toql::sql_builder::SqlBuilder;
    use toql::sql_mapper::MapperOptions;
    use toql::sql_mapper::SqlMapper;

    fn setup_mapper() -> SqlMapper {
        let mut mapper = SqlMapper::new();
       mapper
        .join("author", "JOIN User a ON (id = a.book_id)")
        .map_field_with_options("id", "id", MapperOptions::new().select_always(true).count_query(true))
        .map_field("title", "title")
        .map_field("published", "publishedAt")
        .map_field("author_id", "a.id")
        .map_field("author_username", "a.username")
        ;
        mapper
    }

     #[test]
    fn filter_lk() {
        let mapper = setup_mapper();
        let query = QueryParser::parse("title LK '%Foobar%' ").unwrap(); 

        let result = SqlBuilder::new().build(&mapper, &query).unwrap();
       
        assert_eq!("SELECT id, title, null, null, null FROM Book WHERE title LIKE ?", result.sql_for_table("Book"));
        assert_eq!( "'%Foobar%'", result.where_params.get(0).expect("Parameter expected."));
    }

    #[test]
    fn filter_joined_eq() {
        let mapper = setup_mapper();
        let query = QueryParser::parse("author_id EQ 5").unwrap(); 

        let result = SqlBuilder::new().build(&mapper, &query).unwrap();
       
        assert_eq!("SELECT id, null, null, a.id, null FROM Book JOIN User a ON (id = a.book_id) WHERE a.id = ?", result.sql_for_table("Book"));
        assert_eq!( "5", result.where_params.get(0).expect("Parameter expected."));
    }

     #[test]
    fn filter_having_gt() {
        let mapper = setup_mapper();
        let query = QueryParser::parse("id !GT 5").unwrap(); 

        let result = SqlBuilder::new().build(&mapper, &query).unwrap();
       
        assert_eq!("SELECT id, null, null, null, null FROM Book HAVING id > ?", result.sql_for_table("Book"));
        assert_eq!( "5", result.having_params.get(0).expect("Parameter expected."));
    }