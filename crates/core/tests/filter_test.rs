use std::collections::HashMap;
use toql_core::query::FieldFilter;
use toql_core::query_parser::QueryParser;
use toql_core::sql_builder::SqlBuilder;
use toql_core::sql_builder::SqlBuilderError;
use toql_core::table_mapper::FieldHandler;
use toql_core::table_mapper::FieldOptions;
use toql_core::table_mapper::JoinType;
use toql_core::table_mapper::TableMapper;

fn setup_mapper() -> TableMapper {
    let mut mapper = TableMapper::new("Book");
    mapper
        .join("author", JoinType::Left, "User a", "id = a.book_id")
        .map_field_with_options("id", "id", FieldOptions::new().preselect(true))
        .map_field("title", "title")
        .map_field("published", "publishedAt")
        .map_field("author_id", "a.id")
        .map_field("author_username", "a.username");
    mapper
}

#[test]
fn filter_lk() {
    let mapper = setup_mapper();
    let query = QueryParser::parse("title LK '%Foobar%' ").unwrap();
    println!("{:?}", query);
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, title, null, null, null FROM Book WHERE title LIKE ?",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["%Foobar%"]);
}

#[test]
fn filter_having_gt() {
    let mapper = setup_mapper();
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
    let mapper = setup_mapper();
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
    let mapper = setup_mapper();
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
    let mapper = setup_mapper();
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

    let mut mapper = setup_mapper();
    mapper.alter_handler("title", CustomFieldHandler {});
    let query = QueryParser::parse("title FN MA 'Foobar' ").unwrap();
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT id, title, null, null, null FROM Book WHERE MATCH (title) AGAINST (?)",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["'Foobar'"]);
}
