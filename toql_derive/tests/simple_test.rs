extern crate toql_derive;
use toql_derive::Toql;

#[derive(Debug, Clone)]
#[toql(tables = "PascalCase")]
pub struct Book {
    id: u8,
    title: Option<String>,
    author_id: u8,

    //#[toql(join = "author_id <= id", alias = "a")]
    author: Option<User>,
}

#[derive(Debug, Clone)]
#[toql(tables = "PascalCase")]
pub struct User {
    id: u8, // Always selected

    //#[toql(column = "username", count_filter)]
    username: Option<String>,

    //#[toql(skip)]
    other: String,

    //#[toql(merge = "id <= author_id")]
    books: Vec<Book>,
}

#[test]
fn attributes() {
    let mut mu = toql::sql_mapper::SqlMapper::map::<Book>("book");
    mu.join("author", "LEFT JOIN User a on (b.author_id = a.id)");

    let q = toql::query_parser::QueryParser::parse("id, title, author_id");
    let r = toql::sql_builder::SqlBuilder::new().build(&mu, &q.unwrap());
    assert_eq!("SELECT id, title, author_id FROM User", r.unwrap().to_sql());
}

/* Toql (codegen) */
impl toql::query_builder::FieldsType for Book {
    type FieldsType = BookFields;
}
impl toql::sql_mapper::Map for Book {
    fn insert_new_mapper(
        cache: &mut toql::sql_mapper::SqlMapperCache,
    ) -> &mut toql::sql_mapper::SqlMapper {
        let m = Self::new_mapper("book");
        cache.insert(String::from("Book"), m);
        cache.get_mut("Book").unwrap()
    }
    fn new_mapper(table_alias: &str) -> toql::sql_mapper::SqlMapper {
        let s = format!("{} {}", "Book", table_alias);
        let mut m =
            toql::sql_mapper::SqlMapper::new(if table_alias.is_empty() { "Book" } else { &s });
        Self::map(&mut m, "", table_alias);
        m
    }
    fn map(mapper: &mut toql::sql_mapper::SqlMapper, toql_path: &str, sql_alias: &str) {
        mapper.map_field_with_options(
            &format!(
                "{}{}{}",
                toql_path,
                if toql_path.is_empty() { "" } else { "_" },
                "id"
            ),
            &format!(
                "{}{}{}",
                sql_alias,
                if sql_alias.is_empty() { "" } else { "." },
                "id"
            ),
            toql::sql_mapper::MapperOptions::new().select_always(true),
        );
        mapper.map_field_with_options(
            &format!(
                "{}{}{}",
                toql_path,
                if toql_path.is_empty() { "" } else { "_" },
                "title"
            ),
            &format!(
                "{}{}{}",
                sql_alias,
                if sql_alias.is_empty() { "" } else { "." },
                "title"
            ),
            toql::sql_mapper::MapperOptions::new(),
        );
        mapper.map_field_with_options(
            &format!(
                "{}{}{}",
                toql_path,
                if toql_path.is_empty() { "" } else { "_" },
                "authorId"
            ),
            &format!(
                "{}{}{}",
                sql_alias,
                if sql_alias.is_empty() { "" } else { "." },
                "author_id"
            ),
            toql::sql_mapper::MapperOptions::new().select_always(true),
        );
        mapper.map_join::<User>("author", "a");
        mapper.join(
            "author",
            &format!("LEFT JOIN User a ON ({}.author_id = a.id)", sql_alias),
        );
    }
}
impl Book {
    pub fn fields() -> BookFields {
        BookFields::new()
    }
    pub fn fields_from_path(path: String) -> BookFields {
        BookFields::from_path(path)
    }
}
pub struct BookFields(String);
impl BookFields {
    pub fn new() -> Self {
        Self::from_path(String::from(""))
    }
    pub fn from_path(path: String) -> Self {
        Self(path)
    }
    pub fn id(mut self) -> toql::query::Field {
        self.0.push_str("id");
        toql::query::Field::from(self.0)
    }
    pub fn title(mut self) -> toql::query::Field {
        self.0.push_str("title");
        toql::query::Field::from(self.0)
    }
    pub fn author_id(mut self) -> toql::query::Field {
        self.0.push_str("authorId");
        toql::query::Field::from(self.0)
    }
    pub fn author(mut self) -> <User as toql::query_builder::FieldsType>::FieldsType {
        self.0.push_str("author_");
        <User as toql::query_builder::FieldsType>::FieldsType::from_path(self.0)
    }
}
/* Toql derive (codegen_mysql) */
impl Book {
    pub fn load_path_from_mysql(
        path: &str,
        query: &toql::query::Query,
        mappers: &toql::sql_mapper::SqlMapperCache,
        conn: &mut mysql::Conn,
    ) -> std::vec::Vec<Book> {
        let mapper = mappers.get("Book").unwrap();
        let result = toql::sql_builder::SqlBuilder::new()
            .build_path(path, mapper, &query)
            .unwrap();
        toql::log::info!(
            "SQL = \"{}\" with params {:?}",
            result.to_sql(),
            result.params()
        );
        if result.is_empty() {
            vec![]
        } else {
            let entities_stmt = conn.prep_exec(result.to_sql(), result.params());
            let entities = toql::mysql::row::load::<Book>(entities_stmt).unwrap();
            entities
        }
    }
    pub fn load_dependencies_from_mysql(
        mut _entities: &mut Vec<Book>,
        _query: &mut toql::query::Query,
        _mappers: &toql::sql_mapper::SqlMapperCache,
        _conn: &mut mysql::Conn,
    ) {
    }
}
impl toql::mysql::load::Load<Book> for Book {
    fn load_one(
        mut query: &mut toql::query::Query,
        mappers: &toql::sql_mapper::SqlMapperCache,
        conn: &mut mysql::Conn,
        distinct: bool,
    ) -> Result<Book, toql::load::LoadError> {
        let mapper = mappers.get("Book").unwrap();
        let hint = String::from(if distinct { "DISTINCT" } else { "" });
        let result = toql::sql_builder::SqlBuilder::new()
            .build(mapper, &query)
            .unwrap();
        toql::log::info!(
            "SQL = \"{}\" with params {:?}",
            result.to_sql_for_mysql(&hint, 0, 2),
            result.params()
        );
        let entities_stmt = conn.prep_exec(result.to_sql_for_mysql(&hint, 0, 2), result.params());
        let mut entities = toql::mysql::row::load::<Book>(entities_stmt).unwrap();
        if entities.len() > 1 {
            return Err(toql::load::LoadError::NotUnique);
        } else if entities.is_empty() {
            return Err(toql::load::LoadError::NotFound);
        }
        let entity = entities.get(0).unwrap();
        Book::load_dependencies_from_mysql(&mut entities, &mut query, mappers, conn);
        Ok(entities.pop().unwrap())
    }
    fn load_many(
        mut query: &mut toql::query::Query,
        mappers: &toql::sql_mapper::SqlMapperCache,
        mut conn: &mut mysql::Conn,
        distinct: bool,
        count: bool,
        first: u64,
        max: u16,
    ) -> Result<(std::vec::Vec<Book>, Option<(u32, u32)>), mysql::error::Error> {
        let mapper = mappers.get("Book").unwrap();
        let mut hint = String::from(if count { "SQL_CALC_FOUND_ROWS" } else { "" });
        if distinct {
            if !hint.is_empty() {
                hint.push(' ');
            }
            hint.push_str("DISTINCT");
        }
        let result = toql::sql_builder::SqlBuilder::new()
            .build(mapper, &query)
            .unwrap();
        toql::log::info!(
            "SQL = \"{}\" with params {:?}",
            result.to_sql_for_mysql(&hint, first, max),
            result.params()
        );
        let entities_stmt =
            conn.prep_exec(result.to_sql_for_mysql(&hint, first, max), result.params());
        let mut entities = toql::mysql::row::load::<Book>(entities_stmt).unwrap();
        let mut count_result = None;
        if count {
            toql::log::info!("SQL = \"SELECT FOUND_ROWS();\"");
            let r = conn.query("SELECT FOUND_ROWS();").unwrap();
            let total_count = r.into_iter().next().unwrap().unwrap().get(0).unwrap();
            let result = toql::sql_builder::SqlBuilder::new()
                .build_count(mapper, &query)
                .unwrap();
            toql::log::info!(
                "SQL = \"{}\" with params {:?}",
                result.to_sql_for_mysql("SQL_CALC_FOUND_ROWS", 0, 0),
                result.params()
            );
            conn.prep_exec(
                result.to_sql_for_mysql("SQL_CALC_FOUND_ROWS", 0, 0),
                result.params(),
            )
            .expect("SQL error");
            toql::log::info!("SQL = \"SELECT FOUND_ROWS();\"");
            let r = conn.query("SELECT FOUND_ROWS();").unwrap();
            let filtered_count = r.into_iter().next().unwrap().unwrap().get(0).unwrap();
            count_result = Some((total_count, filtered_count))
        }
        Book::load_dependencies_from_mysql(&mut entities, &mut query, mappers, &mut conn);
        Ok((entities, count_result))
    }
}
impl toql::mysql::row::FromResultRow<Book> for Book {
    fn from_row_with_index(
        mut row: &mut mysql::Row,
        i: &mut usize,
    ) -> Result<Book, mysql::error::Error> {
        Ok(Book {
            id: row.take(*i).unwrap(),
            title: row
                .take({
                    *i += 1;
                    *i
                })
                .unwrap(),
            author_id: row
                .take({
                    *i += 1;
                    *i
                })
                .unwrap(),
            author: if User::is_null(&row, "id") {None} else {Some(<User>::from_row_with_index(&mut row, {
                *i += 1;
                i
            })?)}
        })
    },
   

}
/* Toql (codegen) */
impl toql::query_builder::FieldsType for User {
    type FieldsType = UserFields;
}
impl toql::sql_mapper::Map for User {
    fn insert_new_mapper(
        cache: &mut toql::sql_mapper::SqlMapperCache,
    ) -> &mut toql::sql_mapper::SqlMapper {
        let m = Self::new_mapper("user");
        cache.insert(String::from("User"), m);
        cache.get_mut("User").unwrap()
    }
    fn new_mapper(table_alias: &str) -> toql::sql_mapper::SqlMapper {
        let s = format!("{} {}", "User", table_alias);
        let mut m =
            toql::sql_mapper::SqlMapper::new(if table_alias.is_empty() { "User" } else { &s });
        Self::map(&mut m, "", table_alias);
        m
    }
    fn map(mapper: &mut toql::sql_mapper::SqlMapper, toql_path: &str, sql_alias: &str) {
        mapper.map_field_with_options(
            &format!(
                "{}{}{}",
                toql_path,
                if toql_path.is_empty() { "" } else { "_" },
                "id"
            ),
            &format!(
                "{}{}{}",
                sql_alias,
                if sql_alias.is_empty() { "" } else { "." },
                "id"
            ),
            toql::sql_mapper::MapperOptions::new().select_always(true),
        );
        mapper.map_field_with_options(
            &format!(
                "{}{}{}",
                toql_path,
                if toql_path.is_empty() { "" } else { "_" },
                "username"
            ),
            &format!(
                "{}{}{}",
                sql_alias,
                if sql_alias.is_empty() { "" } else { "." },
                "username"
            ),
            toql::sql_mapper::MapperOptions::new().count_filter(true),
        );
    }
}
impl User {
    pub fn merge_books(t: &mut Vec<User>, o: Vec<Book>) {
        toql::sql_builder::merge(
            t,
            o,
            |t| Option::from(t.id),
            |o| Option::from(o.author_id),
            |t, o| t.books.push(o),
        );
    }
    pub fn fields() -> UserFields {
        UserFields::new()
    }
    pub fn fields_from_path(path: String) -> UserFields {
        UserFields::from_path(path)
    }
}
pub struct UserFields(String);
impl UserFields {
    pub fn new() -> Self {
        Self::from_path(String::from(""))
    }
    pub fn from_path(path: String) -> Self {
        Self(path)
    }
    pub fn id(mut self) -> toql::query::Field {
        self.0.push_str("id");
        toql::query::Field::from(self.0)
    }
    pub fn username(mut self) -> toql::query::Field {
        self.0.push_str("username");
        toql::query::Field::from(self.0)
    }
    pub fn books(mut self) -> <Book as toql::query_builder::FieldsType>::FieldsType {
        self.0.push_str("books_");
        <Book as toql::query_builder::FieldsType>::FieldsType::from_path(self.0)
    }
}
/* Toql derive (codegen_mysql) */
impl User {
    pub fn load_path_from_mysql(
        path: &str,
        query: &toql::query::Query,
        mappers: &toql::sql_mapper::SqlMapperCache,
        conn: &mut mysql::Conn,
    ) -> std::vec::Vec<User> {
        let mapper = mappers.get("User").unwrap();
        let result = toql::sql_builder::SqlBuilder::new()
            .build_path(path, mapper, &query)
            .unwrap();
        toql::log::info!(
            "SQL = \"{}\" with params {:?}",
            result.to_sql(),
            result.params()
        );
        if result.is_empty() {
            vec![]
        } else {
            let entities_stmt = conn.prep_exec(result.to_sql(), result.params());
            let entities = toql::mysql::row::load::<User>(entities_stmt).unwrap();
            entities
        }
    }
    pub fn load_dependencies_from_mysql(
        mut entities: &mut Vec<User>,
        query: &mut toql::query::Query,
        mappers: &toql::sql_mapper::SqlMapperCache,
        conn: &mut mysql::Conn,
    ) {
        let books = Book::load_path_from_mysql("books", &query, mappers, conn);
        User::merge_books(&mut entities, books);
    }
}
impl toql::mysql::load::Load<User> for User {
    fn load_one(
        mut query: &mut toql::query::Query,
        mappers: &toql::sql_mapper::SqlMapperCache,
        conn: &mut mysql::Conn,
        distinct: bool,
    ) -> Result<User, toql::load::LoadError> {
        let mapper = mappers.get("User").unwrap();
        let hint = String::from(if distinct { "DISTINCT" } else { "" });
        let result = toql::sql_builder::SqlBuilder::new()
            .ignore_path("books")
            .build(mapper, &query)
            .unwrap();
        toql::log::info!(
            "SQL = \"{}\" with params {:?}",
            result.to_sql_for_mysql(&hint, 0, 2),
            result.params()
        );
        let entities_stmt = conn.prep_exec(result.to_sql_for_mysql(&hint, 0, 2), result.params());
        let mut entities = toql::mysql::row::load::<User>(entities_stmt).unwrap();
        if entities.len() > 1 {
            return Err(toql::load::LoadError::NotUnique);
        } else if entities.is_empty() {
            return Err(toql::load::LoadError::NotFound);
        }
        let entity = entities.get(0).unwrap();
        query.and(toql::query::Field::from("books_authorId").eq(entity.id));
        User::load_dependencies_from_mysql(&mut entities, &mut query, mappers, conn);
        Ok(entities.pop().unwrap())
    }
    fn load_many(
        mut query: &mut toql::query::Query,
        mappers: &toql::sql_mapper::SqlMapperCache,
        mut conn: &mut mysql::Conn,
        distinct: bool,
        count: bool,
        first: u64,
        max: u16,
    ) -> Result<(std::vec::Vec<User>, Option<(u32, u32)>), mysql::error::Error> {
        let mapper = mappers.get("User").unwrap();
        let mut hint = String::from(if count { "SQL_CALC_FOUND_ROWS" } else { "" });
        if distinct {
            if !hint.is_empty() {
                hint.push(' ');
            }
            hint.push_str("DISTINCT");
        }
        let result = toql::sql_builder::SqlBuilder::new()
            .ignore_path("books")
            .build(mapper, &query)
            .unwrap();
        toql::log::info!(
            "SQL = \"{}\" with params {:?}",
            result.to_sql_for_mysql(&hint, first, max),
            result.params()
        );
        let entities_stmt =
            conn.prep_exec(result.to_sql_for_mysql(&hint, first, max), result.params());
        let mut entities = toql::mysql::row::load::<User>(entities_stmt).unwrap();
        let mut count_result = None;
        if count {
            toql::log::info!("SQL = \"SELECT FOUND_ROWS();\"");
            let r = conn.query("SELECT FOUND_ROWS();").unwrap();
            let total_count = r.into_iter().next().unwrap().unwrap().get(0).unwrap();
            let result = toql::sql_builder::SqlBuilder::new()
                .build_count(mapper, &query)
                .unwrap();
            toql::log::info!(
                "SQL = \"{}\" with params {:?}",
                result.to_sql_for_mysql("SQL_CALC_FOUND_ROWS", 0, 0),
                result.params()
            );
            conn.prep_exec(
                result.to_sql_for_mysql("SQL_CALC_FOUND_ROWS", 0, 0),
                result.params(),
            )
            .expect("SQL error");
            toql::log::info!("SQL = \"SELECT FOUND_ROWS();\"");
            let r = conn.query("SELECT FOUND_ROWS();").unwrap();
            let filtered_count = r.into_iter().next().unwrap().unwrap().get(0).unwrap();
            count_result = Some((total_count, filtered_count))
        }
        query.and(
            toql::query::Field::from("books_authorId")
                .ins(entities.iter().map(|entity| entity.id).collect()),
        );
        User::load_dependencies_from_mysql(&mut entities, &mut query, mappers, &mut conn);
        Ok((entities, count_result))
    }
}

impl User {
    fn is_null(row: &mysql::Row, key: &str) -> bool {
        let v : mysql::Value;
        v = row.get(key).unwrap();
        v == mysql::Value::NULL
    }

}

impl toql::mysql::row::FromResultRow<User> for User {

    fn from_row_with_index(
        mut row: &mut mysql::Row,
        i: &mut usize,
    ) -> Result<User, mysql::error::Error> {
        Ok(User {
            id: row.take(*i).unwrap(),
            username: row
                .take({
                    *i += 1;
                    *i
                })
                .unwrap(),
            books: Vec::new(),
        })
    }
}
