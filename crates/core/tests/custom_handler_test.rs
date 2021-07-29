use std::collections::HashMap;
use toql_core::query::FieldFilter;
use toql_core::query_parser::QueryParser;
use toql_core::sql_builder::SqlBuilder;
use toql_core::sql_builder::SqlBuilderError;
use toql_core::table_mapper::BasicFieldHandler;
use toql_core::table_mapper::FieldHandler;
use toql_core::table_mapper::FieldOptions;
use toql_core::table_mapper::TableMapper;

#[test]
fn custom_handler() {
    struct CustomHandler<T: FieldHandler> {
        base: T,
    }

    impl<T: FieldHandler> FieldHandler for CustomHandler<T> {
        fn build_select(
            &self,
            select: (String, Vec<String>),
            query_params: &HashMap<String, String>,
        ) -> Result<Option<(String, Vec<String>)>, SqlBuilderError> {
            self.base.build_select(select, query_params)
        }

        fn build_filter(
            &self,
            mut select: (String, Vec<String>),
            filter: &FieldFilter,
            params: &HashMap<String, String>,
        ) -> Result<Option<(String, Vec<String>)>, SqlBuilderError> {
            match filter {
                FieldFilter::Fn(name, args) => match name.as_str() {
                    "LN" => {
                        if args.len() != 1 {
                            return Err(SqlBuilderError::FilterInvalid(format!(
                                "filter `{}` expected exactly 1 argument",
                                name
                            )));
                        }
                        select.1.push(args.get(0).unwrap().clone());
                        Ok(Some((format!("LENGTH({}) = ?", select.0), select.1)))
                    }
                    _ => self.base.build_filter(select, filter, params),
                },
                _ => self.base.build_filter(select, filter, params),
            }
        }
        /*  fn build_param(
            &self,
            filter: &FieldFilter,
            params: &HashMap<String, String>,
        ) -> Vec<String> {
            match filter {
                FieldFilter::Fn(name, args) => match name.as_str() {
                    "LN" => args.clone(),
                    _ => self.base.build_param(filter, params),
                },
                _ => self.base.build_param(filter, params),
            }
        } */
        fn build_join(
            &self,
            params: &HashMap<String, String>,
        ) -> Result<Option<String>, SqlBuilderError> {
            self.base.build_join(params)
        }
    }

    let h = CustomHandler {
        base: BasicFieldHandler {},
    };

    let mut mapper = TableMapper::new_with_handler("Book b", h);
    mapper
        .map_field_with_options("id", "b.id", FieldOptions::new().preselect(true))
        .map_field("title", "b.title");

    let query = QueryParser::parse("id GT 2, title FN LN 5").unwrap();
    // println!("{:?}", query);
    let result = SqlBuilder::new().build(&mapper, &query).unwrap();

    assert_eq!(
        "SELECT b.id, b.title FROM Book b WHERE b.id > ? AND LENGTH(b.title) = ?",
        result.to_sql()
    );
    assert_eq!(*result.params(), ["2", "5"]);
}
