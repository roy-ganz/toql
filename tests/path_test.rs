extern crate toql;

    use toql::query_parser::QueryParser;
    use toql::sql_builder::SqlBuilder;
    use toql::sql_mapper::MapperOptions;
    use toql::sql_mapper::SqlMapper;

    fn setup_mapper() -> SqlMapper {
        let mut mapper = SqlMapper::new();
       mapper
        .join("book", "JOIN Book b ON (id = b.id)")
        .map_field_with_options("id", "id", MapperOptions::new().select_always(true).count_query(true))
        .map_field("username", "username")
        .map_field("book_id", "b.id")
     
        ;
        mapper
    }

 #[test]
    fn build_path() {
        let mapper = setup_mapper();
         // build query for path, ignore other fields
         // mapper must provide fields of path
        let query = QueryParser::parse("fooId, bar_id, author_id, author_username, author_book_id EQ 5").unwrap();

        let result = SqlBuilder::new().build_path("author", &mapper, &query).unwrap();
       
        assert_eq!("SELECT id, username, b.id FROM User JOIN Book b ON (id = b.id) WHERE b.id = ?", result.sql_for_table("User"));
    }
 #[test]
      fn ignore_path() {
        let mapper = setup_mapper();
         // build query, ignore path
        let query = QueryParser::parse("id, username, book_id, book_foo").unwrap();

        let result = SqlBuilder::new()
        .ignore_path("book")    // field "book_foo" is not missing, because path "book" in query is ignored, no error is raised
        .build(&mapper, &query).unwrap();
       
        assert_eq!("SELECT id, username FROM User", result.sql_for_table("User"));
    }
