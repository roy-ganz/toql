
    extern crate toql;

    use toql::query_parser::QueryParser;
    use toql::sql_builder::SqlBuilder;
    use toql::sql_mapper::MapperOptions;
    use toql::sql_mapper::SqlMapper;

    fn setup_mapper() -> SqlMapper {
        let mut mapper = SqlMapper::new();
       mapper
        .join("author", "JOIN User a ON (id = a.book_id)")
        .map_field("id", "id", MapperOptions::new().select_always(true).count_query(true))
        .map_field("title", "title", MapperOptions::new())
        .map_field("publishedAt", "published_at", MapperOptions::new())
        .map_field("author_id", "a.id", MapperOptions::new())
        .map_field("author_username", "a.username", MapperOptions::new())
        ;
        mapper
    }

    #[test]
    fn select_wildcard() {
        let mapper = setup_mapper();
        let query = QueryParser::parse("*").unwrap(); // select all top fields

        let result = SqlBuilder::new().build(&mapper, &query).unwrap();
       
        assert_eq!("SELECT id, title, publishedAt, null, null FROM Book", result.sql_for_table("Book"));
    }

    #[test]
    fn select_duplicates() {
        let mapper = setup_mapper();
        let query = QueryParser::parse("id, id, title").unwrap(); // select id twice

        let result = SqlBuilder::new().build(&mapper, &query).unwrap();
       
        assert_eq!("SELECT id, title, null, null, null FROM Book", result.sql_for_table("Book"));
    }

    #[test]
    fn select_optional_join() {
        let mapper = setup_mapper();
        let query = QueryParser::parse("id, title, author_id").unwrap(); // select author from joined table

        let result = SqlBuilder::new().build(&mapper, &query).unwrap();
       
        assert_eq!("SELECT id, title, null, a.id, null FROM Book JOIN User a ON (id = a.book_id)", result.sql_for_table("Book"));
    }

     #[test]
    fn select_hidden() {
        let mapper = setup_mapper();
        let query = QueryParser::parse(".id, .title, publishedAt").unwrap(); // id must always be selected (see mapper), title is hidden

        let result = SqlBuilder::new().build(&mapper, &query).unwrap();
       
        assert_eq!("SELECT id, null, published_at, null, null FROM Book", result.sql_for_table("Book"));
    }

     #[test]
    fn select_missing_field() {
        let mapper = setup_mapper();
        let query = QueryParser::parse("id, fail").unwrap(); // Field fail does not exist in mapper

        let result = SqlBuilder::new().build(&mapper, &query);
       
        assert!(result.is_err(), "Field should be missing.");
    }


    


