
    extern crate toql;

    use toql::query_parser::QueryParser;
    use toql::sql_mapper::BuildOptions;
    use toql::sql_mapper::FieldHandler;
    use toql::sql_mapper::MapperOptions;
    use toql::sql_mapper::SqlMapper;

    fn setup_mapper() -> SqlMapper {
        let mut mapper = SqlMapper::new();
        mapper
            .map_field(
                "id",
                "t1.id",
                MapperOptions::new()
                    .select_always(true)
                    .use_for_count_query(true),
            )
            .map_field("username", "t1.username", MapperOptions::new())
            .map_field("archive", "t1.archive", MapperOptions::new())
            .map_field("age", "t1.age", MapperOptions::new());
        mapper
    }

    #[test]
    fn select_wildcard() {
        let mut mapper = setup_mapper();
        let query = QueryParser::parse("*"); // select all fields

        let result = mapper.build(query, BuildOptions::new());
       
        assert_eq!("t1.id, t1.username, t1.archive, t1.age", result.select_clause);
    }

    #[test]
    fn select_duplicates() {
        let mut mapper = setup_mapper();
        let query = QueryParser::parse("*, id, id"); // select three times id

        let result = mapper.build(query, BuildOptions::new());
       
        assert_eq!("t1.id, t1.username, t1.archive, t1.age", result.select_clause);
    }


