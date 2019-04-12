extern crate toql;

    use toql::query_parser::QueryParser;
    use toql::sql_builder::SqlBuilder;
    use toql::sql_mapper::MapperOptions;
    use toql::sql_mapper::SqlMapper;

    fn setup_mapper() -> SqlMapper {
        let mut mapper = SqlMapper::new("Book");
       mapper
        .map_field_with_options("id", "id", MapperOptions::new().select_always(true).count_query(true))
        .map_field("title", "title")
        .map_field("publishedAt", "published_at")
        ;
        mapper
    }

    #[test]
    fn order_simple() {
        let mapper = setup_mapper();
        
        let query = QueryParser::parse("id, +title, publishedAt").unwrap();

        let result = SqlBuilder::new().build(&mapper, &query).unwrap();
       
        assert_eq!("SELECT id, title, published_at FROM Book ORDER BY title ASC", result.to_sql());
    }
     #[test]
    fn order_priority() {
        let mapper = setup_mapper();
        // Fields can have different ordering priorities: Lower numbers comes first.
        let query = QueryParser::parse("-2id, +title, -3publishedAt").unwrap();

        let result = SqlBuilder::new().build(&mapper, &query).unwrap();
       
        assert_eq!("SELECT id, title, published_at FROM Book ORDER BY title ASC, id DESC, published_at DESC", result.to_sql());
    }

     #[test]
    fn order_natural() {
        let mapper = setup_mapper();
        // + and +1 have the same priority, so order fields according to their appearance
        let query = QueryParser::parse("-1id, +title, -1publishedAt").unwrap();

        let result = SqlBuilder::new().build(&mapper, &query).unwrap();
       
        assert_eq!("SELECT id, title, published_at FROM Book ORDER BY id DESC, title ASC, published_at DESC", result.to_sql());
    }