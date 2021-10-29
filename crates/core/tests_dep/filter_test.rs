use std::collections::HashMap;
use toql_core::{table_mapper_registry::TableMapperRegistry, table_mapper::TableMapper, table_mapper::JoinType, table_mapper::FieldOptions, table_mapper::FieldHandler, sql_builder::SqlBuilderError, sql_builder::SqlBuilder, query_parser::QueryParser, query::FieldFilter};

fn setup_registry() -> TableMapperRegistry {
    let mut book_mapper = TableMapper::new("Book");
    bok_mapper
         .map_join(
            "author",
            "Author",
            JoinType::Inner,
            SqlExpr::literal("Author ").push_other_alias(),
            SqlExpr::aliased_column("author_id").push_literal(" = ").push_other_alias()
        )
        .map_field_with_options("id", "id", FieldOptions::new().preselect(true))
        .map_field("title", "title")
        .map_field("published", "publishedAt");
       

    let mut author_mapper = TableMapper::new("Author");
    author_mapper
          .map_field("id", "a.id")
            .map_field("username", "a.username"); 
    
    let mut registry= TableMapperRegistry::new();
    registry.insert(book_mapper);
    registry.insert(author_mapper);

    registry
}

#[test]
fn filter_lk() {
    let registry = setup_registry();
    let query = QueryParser::parse("title LK '%Foobar%' ").expect("Invalid Toql query");
    
    let result = SqlBuilder::new("book", &registry)
            .build_select("", &query).expect("Unable to build SQL.");

    assert_eq!(
        "SELECT id, title FROM Book WHERE title LIKE ?",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["%Foobar%"]);
}

#[test]
fn filter_having_gt() {
    let mapper = setup_registry();
    let query = QueryParser::parse("id !GT 5").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, null, null, null, null FROM Book HAVING id > ?",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["5"]);
}

#[test]
fn filter_bw() {
    let mapper = setup_registry();
    let query = QueryParser::parse("id BW 0 5").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, null, null, null, null FROM Book WHERE id BETWEEN ? AND ?",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["0", "5"]);
}
#[test]
fn filter_in() {
    let mapper = setup_registry();
    let query = QueryParser::parse("id IN 0 1 5").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, null, null, null, null FROM Book WHERE id IN (?,?,?)",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["0", "1", "5"]);
}

#[test]
fn filter_joined_eq() {
    let mapper = setup_registry();
    let query = QueryParser::parse("author_id EQ 5").unwrap();

    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!("SELECT id, null, null, a.id, null FROM Book JOIN User a ON (id = a.book_id) WHERE a.id = ?", result.to_sql());
    assert_eq!(*result.params(), ["5"]);
}

#[test]
fn filter_fnc() {
    struct CustomFieldHandler {};
    impl FieldHandler for CustomFieldHandler {
        fn build_select(
            &self,
            select: (String, Vec<String>),
            _params: &HashMap<String, String>,
        ) -> Result<Option<(String, Vec<String>)>, SqlBuilderError> {
            Ok(Some(select))
        }
        fn build_filter(
            &self,
            mut select: (String, Vec<String>),
            filter: &FieldFilter,
            _params: &HashMap<String, String>,
        ) -> Result<Option<(String, Vec<String>)>, SqlBuilderError> {
            match filter {
                FieldFilter::Fn(name, args) => match (*name).as_ref() {
                    "MA" => {
                        if args.len() != 1 {
                            return Err(SqlBuilderError::FilterInvalid(format!(
                                "filter `{}` expected exactly 1 argument",
                                name
                            )));
                        }
                        select.1.push(args.get(0).unwrap().clone());
                        Ok(Some((
                            format!("MATCH ({}) AGAINST (?)", select.0),
                            select.1,
                        )))
                    }
                    _ => Ok(None),
                },
                _ => Ok(None),
            }
        }
        /* fn build_param(
            &self,
            filter: &FieldFilter,
            _params: &HashMap<String, String>,
        ) -> Vec<String> {
            match filter {
                FieldFilter::Fn(name, args) => match (*name).as_ref() {
                    "MA" => {
                        if args.len() != 1 {
                            vec![String::new()]
                        } else {
                            args.clone()
                        }
                    }
                    _ => vec![],
                },
                _ => vec![],
            }
        } */
    }

    let mut mapper = setup_registry();
    mapper.alter_handler("title", CustomFieldHandler {});
    let query = QueryParser::parse("title FN MA 'Foobar' ").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, title, null, null, null FROM Book WHERE MATCH (title) AGAINST (?)",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["'Foobar'"]);
}
