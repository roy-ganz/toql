use toql::{
    mock_db::MockDb,
    prelude::{fields, paths, query, sql_expr, Cache, Join, Toql, ToqlApi},
};
use tracing_test::traced_test;

#[derive(Debug, Default)]
pub struct Level1 {
    id: u64,
    level2: Join<Level2>, // column `level_2`, part of this primary key
    text: String,
}

#[derive(Debug, Default)]
pub struct Level2 {
    id: u64,
    level3: Join<Level3>, // column `level_3`, part of this primary key
    text: String,
}

#[derive(Debug, Default)]
pub struct Level3 {
    id: u64,
    text: String,
}
/*
#[derive(Debug, Default, Toql)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    #[toql(key, join(columns(self="level2_id", other ="id")))]
    level2: Join<Level2>, // column `level_2`, part of this primary key
    text: String,
}

#[derive(Debug, Default, Toql)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    #[toql(key, join(columns(self="level3_id", other ="id")))]
    level3: Join<Level3>,  // column `level_3`, part of this primary key
    text: String,
}

#[derive(Debug, Default, Toql)]
pub struct Level3 {
    #[toql(key)]
    id: u64,
    text: String,
}
*/
fn populated_level() -> Level1 {
    let l3 = Level3 {
        id: 3,
        text: "level3".to_string(),
    };
    let l2 = Level2 {
        id: 2,
        level3: Join::with_entity(l3),
        text: "level2".to_string(),
    };

    Level1 {
        id: 1,
        level2: Join::with_entity(l2),
        text: "level1".to_string(),
    }
}

#[tokio::test]
#[traced_test("info")]
async fn load() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load level 1 + level2 + level3 (key join)
    let q = query!(Level1, "*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id, level1_level2_level3.id, level1_level2_level3.text, level1_level2.text, level1.text \
            FROM Level1 level1 \
            JOIN (Level2 level1_level2 \
                JOIN (Level3 level1_level2_level3) \
                ON (level1_level2.level3_id = level1_level2_level3.id)) \
            ON (level1.level2_id = level1_level2.id AND level1.level2_level3_id = level1_level2.level3_id)"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn delete() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Delete without filter return no queries
    // This is for safety, otherwise everything would be deleted
    let q = query!(Level1, "*"); // Query contains no filter
    assert!(toql.delete_many(q).await.is_ok());
    assert_eq!(toql.sqls_empty(), true);

    // Delete with filter on level1
    let q = query!(Level1, "id eq 4"); // Query contains filter
    assert!(toql.delete_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "DELETE level1 FROM Level1 level1 WHERE level1.id = 4"
    );

    // Delete with filter on level 5
    let q = query!(Level1, "level2_level3_id eq 5");
    assert!(toql.delete_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sql(),
           "DELETE level1 FROM Level1 level1 \
                JOIN (Level2 level1_level2 \
                    JOIN (Level3 level1_level2_level3) \
                    ON (level1_level2.level3_id = level1_level2_level3.id)) \
                    ON (level1.level2_id = level1_level2.id AND level1.level2_level3_id = level1_level2.level3_id) \
            WHERE level1_level2_level3.id = 5");
}

#[tokio::test]
#[traced_test("info")]
async fn insert() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    let mut l = Level1::default();

    // insert level 1
    assert!(toql.insert_one(&mut l, paths!(top)).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "INSERT INTO Level1 (id, level2_id, level2_level3_id, text) VALUES (0, 0, 0, '') WRONG! -> (id, level2_id, text)"
    );

    // insert path levels 1..5
    // this will only insert level 1 + 2,
    // level 3.. is skipped (unselected)
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3"))
        .await
        .is_ok());
    let mut sqls = toql.take_unsafe_sqls();
    sqls.sort();
    assert_eq!(
        sqls,
        [
            "INSERT INTO Level1 (id, level2_id, level2_level3_id, text) VALUES (0, 0, 0, '')",
            "INSERT INTO Level2 (id, level3_id, text) VALUES (0, 0, '')",
            "INSERT INTO Level3 (id, text) VALUES (0, '')",
        ]
    );

    // insert path levels 1..3
    // this will insert level 1..3
    let mut l = populated_level();
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3"))
        .await
        .is_ok());
    let mut sqls = toql.take_unsafe_sqls();
    sqls.sort();
    assert_eq!(
        sqls,
        [
            "INSERT INTO Level1 (id, level2_id, level2_level3_id, text) VALUES (1, 2, 3, 'level1')",
            "INSERT INTO Level2 (id, level3_id, text) VALUES (2, 3, 'level2')",
            "INSERT INTO Level3 (id, text) VALUES (3, 'level3')"
        ]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn update() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Update level 1
    // Nothing is updated, fields are empty
    let mut l1 = Level1::default();
    assert!(toql.update_one(&mut l1, fields!(top)).await.is_ok());
    assert_eq!(toql.sqls_empty(), true);

    // Update level 1 with invalid key
    // Nothing is updated
    let mut l1 = populated_level();
    l1.id = 0;
    assert!(toql.update_one(&mut l1, fields!(top)).await.is_ok());
    assert_eq!(toql.sqls_empty(), true);

    // Update level 1 (text + foreign key)
    let mut l1 = populated_level();
    assert!(toql.update_one(&mut l1, fields!(top)).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level1 SET text = 'level1' WHERE id = 1 AND level2_id = 2 AND level2_level3_id = 3 JOIN ... TODO"
    );
    /*
    // Update level 5 (text)
    let mut l1 = populated_level();
    assert!(toql
        .update_one(&mut l1, fields!(Level1, "level2_level3_level4_level5_*"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level5 SET text = 'level5' WHERE id = 5"
    );

    // Update level 1 - 5
    let mut l1 = populated_level();
    assert!(toql
        .update_one(
            &mut l1,
            fields!(
                Level1,
                "*, level2_*, \
            level2_level3_*, level2_level3_level4_*,\
            level2_level3_level4_level5_*"
            ),
        )
        .await
        .is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
        [
            "UPDATE Level1 SET text = \'level1\', level2_id = 2 WHERE id = 1",
            "UPDATE Level2 SET text = \'level2\', level_3 = 3 WHERE id = 2",
            "UPDATE Level3 SET text = \'level3\', level4_id = 4 WHERE id = 3",
            "UPDATE Level4 SET text = \'level4\', level5_id = 5 WHERE id = 4",
            "UPDATE Level5 SET text = \'level5\' WHERE id = 5"
        ]
    );
    */
}

impl toql::toql_api::insert::Insert for Level1 where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::toql_api::insert::Insert for &mut Level1 where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::toql_api::update::Update for Level1 where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::toql_api::update::Update for &mut Level1 where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::toql_api::delete::Delete for Level1 where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap
{
}
impl toql::toql_api::delete::Delete for &Level1 where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap
{
}
impl<R, E> toql::toql_api::load::Load<R, E> for Level1
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl<R, E> toql::toql_api::load::Load<R, E> for &Level1
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl toql::toql_api::count::Count for Level1 where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped
{
}
impl toql::toql_api::count::Count for &Level1 where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped
{
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Level1Key {
    pub id: u64,
    pub level2: <Join<Level2> as toql::keyed::Keyed>::Key,
}
impl toql::key_fields::KeyFields for Level1Key {
    type Entity = Level1;
    fn fields() -> Vec<String> {
        let mut fields: Vec<String> = Vec::new();
        fields.push(String::from("id"));
        <<Join<Level2> as toql::keyed::Keyed>::Key as toql::key_fields::KeyFields>::fields()
            .iter()
            .for_each(|other_field| {
                fields.push(format!("{}_{}", "level2", other_field));
            });
        fields
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params.extend_from_slice(&toql::key::Key::params(&key.level2));
        params
    }
}
impl toql::key_fields::KeyFields for &Level1Key {
    type Entity = Level1;
    fn fields() -> Vec<String> {
        <Level1Key as toql::key_fields::KeyFields>::fields()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <Level1Key as toql::key_fields::KeyFields>::params(self)
    }
}
impl toql::key::Key for Level1Key {
    type Entity = Level1;
    fn columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("id"));
        <<Join<Level2> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
            .iter()
            .for_each(|other_column| {
                let default_self_column = format!("level2_{}", other_column);
                let column = {
                    if cfg!(debug_assertions) {
                        let valid_columns =
                            <<Level2 as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                        let invalid_columns: Vec<String> = ["id"]
                            .iter()
                            .filter(|col| !valid_columns.iter().any(|s| &s == col))
                            .map(|c| c.to_string())
                            .collect::<Vec<_>>();
                        if !invalid_columns.is_empty() {
                            toql::tracing::warn!(
                                "On `{}::{}` invalid columns found: `{}`. Valid columns are: `{}`",
                                "Level1",
                                "level2",
                                invalid_columns.join(","),
                                valid_columns.join(",")
                            );
                        }
                    }
                    let self_column = match other_column.as_str() {
                        "id" => "level2_id",
                        _ => &default_self_column,
                    };
                    self_column
                };
                columns.push(column.to_string());
            });
        columns
    }
    fn default_inverse_columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("level1_id"));
        <<Join<Level2> as toql::keyed::Keyed>::Key as toql::key::Key>::default_inverse_columns()
            .iter()
            .for_each(|other_column| {
                let default_self_column = format!("level2_{}", other_column);
                let column = {
                    if cfg!(debug_assertions) {
                        let valid_columns =
                            <<Level2 as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                        let invalid_columns: Vec<String> = ["id"]
                            .iter()
                            .filter(|col| !valid_columns.iter().any(|s| &s == col))
                            .map(|c| c.to_string())
                            .collect::<Vec<_>>();
                        if !invalid_columns.is_empty() {
                            toql::tracing::warn!(
                                "On `{}::{}` invalid columns found: `{}`. Valid columns are: `{}`",
                                "Level1",
                                "level2",
                                invalid_columns.join(","),
                                valid_columns.join(",")
                            );
                        }
                    }
                    let self_column = match other_column.as_str() {
                        "id" => "level2_id",
                        _ => &default_self_column,
                    };
                    self_column
                };
                columns.push(column.to_string());
            });
        columns
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params.extend_from_slice(&toql::key::Key::params(&key.level2));
        params
    }
}
impl toql::key::Key for &Level1Key {
    type Entity = Level1;
    fn columns() -> Vec<String> {
        <Level1Key as toql::key::Key>::columns()
    }
    fn default_inverse_columns() -> Vec<String> {
        <Level1Key as toql::key::Key>::default_inverse_columns()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <Level1Key as toql::key::Key>::params(self)
    }
}
impl From<Level1Key> for toql::sql_arg::SqlArg {
    fn from(t: Level1Key) -> toql::sql_arg::SqlArg {
        toql::sql_arg::SqlArg::from(t.id)
    }
}
impl From<&Level1Key> for toql::sql_arg::SqlArg {
    fn from(t: &Level1Key) -> toql::sql_arg::SqlArg {
        toql::sql_arg::SqlArg::from(t.id.to_owned())
    }
}
impl toql::keyed::Keyed for Level1 {
    type Key = Level1Key;
    fn key(&self) -> Self::Key {
        Level1Key {
            id: self.id.to_owned(),
            level2: <Join<Level2> as toql::keyed::Keyed>::key(&self.level2),
        }
    }
}
impl toql::keyed::Keyed for &Level1 {
    type Key = Level1Key;
    fn key(&self) -> Self::Key {
        <Level1 as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::Keyed for &mut Level1 {
    type Key = Level1Key;
    fn key(&self) -> Self::Key {
        <Level1 as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::KeyedMut for Level1 {
    fn set_key(&mut self, key: Self::Key) {
        self.id = key.id;
        <Join<Level2> as toql::keyed::KeyedMut>::set_key(&mut self.level2, key.level2);
    }
}
impl toql::keyed::KeyedMut for &mut Level1 {
    fn set_key(&mut self, key: Self::Key) {
        <Level1 as toql::keyed::KeyedMut>::set_key(self, key)
    }
}
impl std::convert::TryFrom<Vec<toql::sql_arg::SqlArg>> for Level1Key {
    type Error = toql::error::ToqlError;
    fn try_from(args: Vec<toql::sql_arg::SqlArg>) -> toql::result::Result<Self> {
        use std::convert::TryInto;
        Ok(Level1Key {
            id: args
                .get(0)
                .ok_or(toql::error::ToqlError::ValueMissing("id".to_string()))?
                .try_into()?,
            level2: <Join<Level2> as toql::keyed::Keyed>::Key::try_from(Vec::from(&args[1..]))?,
        })
    }
}
impl std::convert::From<(u64, <Level2 as toql::keyed::Keyed>::Key)> for Level1Key {
    fn from(key: (u64, <Level2 as toql::keyed::Keyed>::Key)) -> Self {
        Self {
            id: key.0,
            level2: key.1,
        }
    }
}
impl std::hash::Hash for Level1 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        <Level1 as toql::keyed::Keyed>::key(self).hash(state);
    }
}
impl toql::identity::Identity for Level1 {
    fn columns() -> Vec<String> {
        let mut columns = Vec::with_capacity(1usize);
        columns.push(String::from("id"));
        columns
    }
    fn set_column(
        &mut self,
        column: &str,
        value: &toql::sql_arg::SqlArg,
    ) -> toql::result::Result<()> {
        use std::convert::TryInto;
        match column {
            "id" => self.id = value.try_into()?,
            _ => {}
        }
        Ok(())
    }
}

impl toql::table_mapper::mapped::Mapped for Level1 {
    fn type_name() -> String {
        String::from("Level1")
    }
    fn table_name() -> String {
        String::from("Level1")
    }
    fn table_alias() -> String {
        String::from("level1")
    }
    #[allow(redundant_semicolons)]
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        mapper.map_column_with_options(
            "id",
            "id",
            toql::table_mapper::field_options::FieldOptions::new()
                .key(true)
                .preselect(true),
        );
        mapper .
        map_join_with_options("level2", "Level2", toql :: table_mapper ::
                              join_type :: JoinType :: Inner,
                              {
                                  let mut t = toql :: sql_expr :: SqlExpr ::
                                  literal(< Level2 as toql :: table_mapper ::
                                          mapped :: Mapped > :: table_name())
                                  ; t . push_literal(" ") ; t .
                                  push_other_alias() ; t
                              },
                              {
                                  let mut t = toql :: sql_expr :: SqlExpr ::
                                  new() ; [String :: from("id")] . iter() .
                                  for_each(| other_column |
                                           {
                                               let default_self_column =
                                               format !
                                               ("level2_{}", other_column) ; ;
                                               let self_column =
                                               {
                                                   if cfg ! (debug_assertions)
                                                   {
                                                       let valid_columns = <<
                                                       Level2 as toql :: keyed
                                                       :: Keyed > :: Key as
                                                       toql :: key :: Key > ::
                                                       columns() ; let
                                                       invalid_columns : Vec <
                                                       String > = ["id"] .
                                                       iter() .
                                                       filter(| col | !
                                                              valid_columns .
                                                              iter() .
                                                              any(| s | & s ==
                                                                  col)) .
                                                       map(| c | c .
                                                           to_string()) .
                                                       collect :: < Vec < _ >>
                                                       () ; if !
                                                       invalid_columns .
                                                       is_empty()
                                                       {
                                                           toql :: tracing ::
                                                           warn !
                                                           ("On `{}::{}` invalid columns found: `{}`. Valid columns are: `{}`",
                                                            "Level1",
                                                            "level2",
                                                            invalid_columns .
                                                            join(","),
                                                            valid_columns .
                                                            join(",")) ;
                                                       }
                                                   } let self_column = match
                                                   other_column . as_str()
                                                   {
                                                       "id" => "level2_id", _
                                                       => &
                                                       default_self_column
                                                   } ; self_column
                                               } ; t . push_self_alias() ; t .
                                               push_literal(".") ; t .
                                               push_literal(self_column) ; t .
                                               push_literal(" = ") ; t .
                                               push_other_alias() ; t .
                                               push_literal(".") ; t .
                                               push_literal(other_column) ; t
                                               . push_literal(" AND ") ;
                                           }) ; t . pop_literals(5) ; t
                              }, toql :: table_mapper :: join_options ::
                              JoinOptions :: new() . preselect(true) .
                              key(true)) ;
        mapper.map_column_with_options(
            "text",
            "text",
            toql::table_mapper::field_options::FieldOptions::new().preselect(true),
        );
        Ok(())
    }
}
impl toql::table_mapper::mapped::Mapped for &Level1 {
    fn type_name() -> String {
        <Level1 as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Level1 as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Level1 as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Level1 as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}
impl toql::table_mapper::mapped::Mapped for &mut Level1 {
    fn type_name() -> String {
        <Level1 as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Level1 as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Level1 as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Level1 as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Level1Key
where
    E: std::convert::From<toql::error::ToqlError>,
    u64: toql::from_row::FromRow<R, E>,
    Level2: toql::from_row::FromRow<R, E> + toql::keyed::Keyed,
    <Level2 as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(
            0 + <u64 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                + <Level2 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?,
        )
    }
    #[allow(unused_variables, unused_mut)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Level1Key>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        Ok(Some(Level1Key {
            id: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Level1Key::id".to_string(),
                    )
                    .into(),
                )?
            },
            level2: {
                <<Join<Level2> as toql::keyed::Keyed>::Key>::from_row(row, i, iter)?.ok_or(
                    toql::error::ToqlError::ValueMissing("Level1Key::level2".to_string()),
                )?
            },
        }))
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Level1
where
    E: std::convert::From<toql::error::ToqlError>,
    String: toql::from_row::FromRow<R, E>,
    u64: toql::from_row::FromRow<R, E>,
    Level2: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(
            0 + <u64 as toql::from_row::FromRow<_, E>>::forward(&mut iter)?
                + if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Level2 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                } else {
                    0
                }
                + <String as toql::from_row::FromRow<_, E>>::forward(&mut iter)?,
        )
    }
    #[allow(unused_variables, unused_mut, unused_imports)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Level1>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        use toql::sql_builder::select_stream::Select;
        Ok(Some(Level1 {
            id: {
                match <u64 as toql::from_row::FromRow<_, E>>::from_row(row, i, iter)? {
                    Some(s) => s,
                    _ => return Ok(None),
                }
            },
            level2: {
                if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    match toql::from_row::FromRow::<_, E>::from_row(row, i, iter)? {
                        Some(s) => s,
                        None => {
                            return Err(
                                toql::error::ToqlError::ValueMissing("level2".to_string()).into()
                            )
                        }
                    }
                } else {
                    return Err(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::SelectionExpected(
                            "level2".to_string(),
                        ),
                    )
                    .into());
                }
            },
            text: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Level1::text".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl toql::query_fields::QueryFields for Level1 {
    type FieldsType = Level1Fields;
    fn fields() -> Level1Fields {
        Level1Fields::new()
    }
    fn fields_from_path(path: String) -> Level1Fields {
        Level1Fields::from_path(path)
    }
}
pub struct Level1Fields(String);
impl toql::query_path::QueryPath for Level1Fields {
    fn into_path(self) -> String {
        self.0
    }
}
impl Level1Fields {
    pub fn new() -> Self {
        Self::from_path(String::from(""))
    }
    pub fn from_path(path: String) -> Self {
        Self(path)
    }
    pub fn into_name(self) -> String {
        self.0
    }
    pub fn id(mut self) -> toql::query::field::Field {
        self.0.push_str("id");
        toql::query::field::Field::from(self.0)
    }
    pub fn level2(mut self) -> <Level2 as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("level2_");
        <Level2 as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
    pub fn text(mut self) -> toql::query::field::Field {
        self.0.push_str("text");
        toql::query::field::Field::from(self.0)
    }
}

impl toql::tree::tree_insert::TreeInsert for Level1 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        let mut e = toql::sql_expr::SqlExpr::new();
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level2" => {
                    return Ok(<Level2 as toql::tree::tree_insert::TreeInsert>::columns(
                        descendents,
                    )?);
                }
                f @ _ => {
                    return Err(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        )
                        .into(),
                    );
                }
            },
            None => {
                e.push_literal("(");
                e.push_literal("id");
                e.push_literal(", ");
                for other_column in
                    <<Level2 as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("level2_{}", other_column);
                   
                    e.push_literal(default_self_column);
                    e.push_literal(", ");
                }
                e.push_literal("text");
                e.push_literal(", ");
                e.pop_literals(2);
                e.push_literal(")");
            }
        }
        Ok(e)
    }
    #[allow(unused_mut, unused_variables)]
    fn values<'a, 'b, I, J>(
        &self,
        mut descendents: I,
        roles: &std::collections::HashSet<String>,
        mut should_insert: &mut J,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
        J: Iterator<Item = &'b bool>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level2" => toql::tree::tree_insert::TreeInsert::values(
                    &self.level2,
                    descendents,
                    roles,
                    should_insert,
                    values,
                )?,
                f @ _ => {
                    return Err(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        )
                        .into(),
                    );
                }
            },
            None => {
                if !*should_insert.next().unwrap_or(&false) {
                    return Ok(());
                }
                values.push_literal("(");
                values.push_arg(toql::sql_arg::SqlArg::from(&self.id));
                values.push_literal(", ");
                
                let params= &toql::key::Key::params(&toql::keyed::Keyed::key(&self.level2));

                let other_columns =  <<Level2 as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                for mapped_other_column in ["id"].iter() {
                    let n = other_columns.iter().position(|c| c == *mapped_other_column);
                    if let Some(n) = n {
                        let p = params.get(n).ok_or(
                        toql::error::ToqlError::ValueMissing(
                            <Level2Key as toql::key::Key>::columns().join(", "),
                        ))?;
                        values.push_arg(p.to_owned());
                        values.push_literal(", ");
                    }
                }

                
              
                values.push_arg(toql::sql_arg::SqlArg::from(&self.text));
                values.push_literal(", ");
                values.pop_literals(2);
                values.push_literal("), ");
            }
        }
        Ok(())
    }
}
impl toql::tree::tree_insert::TreeInsert for &Level1 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level1 as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, 'b, I, J>(
        &self,
        mut descendents: I,
        roles: &std::collections::HashSet<String>,
        mut should_insert: &mut J,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
        J: Iterator<Item = &'b bool>,
    {
        <Level1 as toql::tree::tree_insert::TreeInsert>::values(
            self,
            descendents,
            roles,
            should_insert,
            values,
        )
    }
}
impl toql::tree::tree_insert::TreeInsert for &mut Level1 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level1 as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, 'b, I, J>(
        &self,
        mut descendents: I,
        roles: &std::collections::HashSet<String>,
        mut should_insert: &mut J,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
        J: Iterator<Item = &'b bool>,
    {
        <Level1 as toql::tree::tree_insert::TreeInsert>::values(
            self,
            descendents,
            roles,
            should_insert,
            values,
        )
    }
}

impl toql::tree::tree_update::TreeUpdate for Level1 {
    #[allow(unused_mut, unused_variables, unused_parens)]
    fn update<'a, I>(
        &self,
        mut descendents: I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                f @ _ => {
                    return Err(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        )
                        .into(),
                    );
                }
            },
            None => {
                let key = <Self as toql::keyed::Keyed>::key(&self);
                if !toql::sql_arg::valid_key(&toql::key::Key::params(&key)) {
                    return Ok(());
                }
                let path_selected = fields.contains("*");
                let mut expr = toql::sql_expr::SqlExpr::new();
                expr.push_literal("UPDATE ");
                expr.push_literal("Level1");
                expr.push_literal(" SET ");
                let tokens = expr.tokens().len();
                if ((path_selected) || fields.contains("text")) {
                    expr.push_literal("text");
                    expr.push_literal(" = ");
                    expr.push_arg(toql::sql_arg::SqlArg::from(&self.text));
                    expr.push_literal(", ");
                }
                expr.pop();
                if expr.tokens().len() > tokens {
                    expr.push_literal(" WHERE ");
                    let key = <Self as toql::keyed::Keyed>::key(&self);
                    let resolver =
                        toql::sql_expr::resolver::Resolver::new().with_self_alias("Level1");
                    expr.extend(
                        resolver.alias_to_literals(&toql::key::Key::predicate_expr(&key, false))?,
                    );
                    exprs.push(expr);
                }
            }
        };
        Ok(())
    }
}
impl toql::tree::tree_update::TreeUpdate for &mut Level1 {
    #[allow(unused_mut)]
    fn update<'a, I>(
        &self,
        mut descendents: I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level1 as toql::tree::tree_update::TreeUpdate>::update(
            self,
            descendents,
            fields,
            roles,
            exprs,
        )
    }
}

impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for Level1
where
    E: std::convert::From<toql::error::ToqlError>,
    Level1Key: toql::from_row::FromRow<R, E>,
    Level2: toql::tree::tree_index::TreeIndex<R, E>,
{
    #[allow(unused_variables, unused_mut)]
    fn index<'a, I>(
        mut descendents: I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        use toql::from_row::FromRow;
        use toql::sql_builder::select_stream::Select;
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level2" => <Level2 as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    descendents,
                    rows,
                    row_offset,
                    index,
                )?,
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    )
                    .into());
                }
            },
            None => {
                for (n, row) in rows.into_iter().enumerate() {
                    let mut iter = std::iter::repeat(&Select::Query);
                    let mut i = row_offset;
                    let fk = Level1Key::from_row(&row, &mut i, &mut iter)?.ok_or(
                        toql::error::ToqlError::ValueMissing(
                            <Level1Key as toql::key::Key>::columns().join(", "),
                        ),
                    )?;
                    let mut s = DefaultHasher::new();
                    fk.hash(&mut s);
                    let fk_hash = s.finish();
                    index
                        .entry(fk_hash)
                        .and_modify(|h| h.push(n))
                        .or_insert(vec![n]);
                }
            }
        }
        Ok(())
    }
}
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for &Level1
where
    E: std::convert::From<toql::error::ToqlError>,
    Level1Key: toql::from_row::FromRow<R, E>,
    Level2: toql::tree::tree_index::TreeIndex<R, E>,
{
    #[allow(unused_mut)]
    fn index<'a, I>(
        mut descendents: I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level1 as toql::tree::tree_index::TreeIndex<R, E>>::index(
            descendents,
            rows,
            row_offset,
            index,
        )
    }
}

impl toql::tree::tree_identity::TreeIdentity for Level1 {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level2" => {
                    Ok(<Level2 as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)?)
                }
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    )
                    .into());
                }
            },
            None => Ok(false),
        }
    }
    #[allow(unused_variables, unused_mut)]
    fn set_id<'a, 'b, I>(
        &mut self,
        mut descendents: I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level2" => toql::tree::tree_identity::TreeIdentity::set_id(
                    &mut self.level2,
                    descendents,
                    action,
                )?,
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    )
                    .into());
                }
            },
            None => {
                fn set_self_key(
                    entity: &mut Level1,
                    args: &mut Vec<toql::sql_arg::SqlArg>,
                    invalid_only: bool,
                ) -> std::result::Result<(), toql::error::ToqlError> {
                    if invalid_only {
                        let self_key = toql::keyed::Keyed::key(&entity);
                        let self_key_params = toql::key::Key::params(&self_key);
                        if toql::sql_arg::valid_key(&self_key_params) {
                            return Ok(());
                        }
                    }
                    let n =
                        <<Level1 as toql::keyed::Keyed>::Key as toql::key::Key>::columns().len();
                    let end = args.len();
                    let args: Vec<toql::sql_arg::SqlArg> =
                        args.drain(end - n..).collect::<Vec<_>>();
                    let key = std::convert::TryFrom::try_from(args)?;
                    toql::keyed::KeyedMut::set_key(entity, key);
                    Ok(())
                }
                if let toql::tree::tree_identity::IdentityAction::SetInvalid(args) = action {
                    set_self_key(self, &mut args.borrow_mut(), true)?;
                }
                if let toql::tree::tree_identity::IdentityAction::Set(args) = action {
                    set_self_key(self, &mut args.borrow_mut(), false)?;
                }
                let self_key = toql::keyed::Keyed::key(&self);
                let self_key_params = toql::key::Key::params(&self_key);
                let self_key_columns =
                    <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
            }
        }
        Ok(())
    }
}
impl toql::tree::tree_identity::TreeIdentity for &mut Level1 {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level1 as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)
    }
    #[allow(unused_mut)]
    fn set_id<'a, 'b, I>(
        &mut self,
        mut descendents: I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        toql::tree::tree_identity::TreeIdentity::set_id(*self, descendents, action)
    }
}

impl toql::tree::tree_map::TreeMap for Level1 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        if registry.get("Level1").is_none() {
            registry.insert_new_mapper::<Level1>()?;
        }
        <Level2 as toql::tree::tree_map::TreeMap>::map(registry)?;
        Ok(())
    }
}
impl toql::tree::tree_map::TreeMap for &Level1 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        <Level1 as toql::tree::tree_map::TreeMap>::map(registry)
    }
}
impl toql::tree::tree_map::TreeMap for &mut Level1 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        <Level1 as toql::tree::tree_map::TreeMap>::map(registry)
    }
}

impl toql::tree::tree_predicate::TreePredicate for Level1 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        Ok(match descendents.next() {
            Some(d) => match d.as_str() {
                "level2" => {
                    <Level2 as toql::tree::tree_predicate::TreePredicate>::columns(descendents)?
                }
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    ));
                }
            },
            None => <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns(),
        })
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level2" => toql::tree::tree_predicate::TreePredicate::args(
                    &self.level2,
                    descendents,
                    args,
                )?,
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    ));
                }
            },
            None => {
                let key = <Self as toql::keyed::Keyed>::key(&self);
                args.extend(<<Self as toql::keyed::Keyed>::Key as toql::key::Key>::params(&key));
            }
        }
        Ok(())
    }
}
impl toql::tree::tree_predicate::TreePredicate for &Level1 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level1 as toql::tree::tree_predicate::TreePredicate>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level1 as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl toql::tree::tree_predicate::TreePredicate for &mut Level1 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level1 as toql::tree::tree_predicate::TreePredicate>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level1 as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}

impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for Level1
where
    E: std::convert::From<toql::error::ToqlError>,
    Level1Key: toql::from_row::FromRow<R, E>,
    Level2: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
{
    #[allow(unreachable_code, unused_variables, unused_mut, unused_imports)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        use toql::from_row::FromRow;
        use toql::keyed::Keyed;
        use toql::sql_builder::select_stream::Select;
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level2" => toql::tree::tree_merge::TreeMerge::merge(
                    &mut self.level2,
                    descendents,
                    &field,
                    rows,
                    row_offset,
                    index,
                    selection_stream,
                )?,
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    )
                    .into());
                }
            },
            None => {
                let pk: Level1Key = <Self as toql::keyed::Keyed>::key(&self);
                let mut s = DefaultHasher::new();
                pk.hash(&mut s);
                let h = s.finish();
                let default_vec: Vec<usize> = Vec::new();
                let row_numbers: &Vec<usize> = index.get(&h).unwrap_or(&default_vec);
                let n = row_offset;
                match field {
                    f @ _ => {
                        return Err(toql::error::ToqlError::SqlBuilderError(
                            toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                                f.to_string(),
                            ),
                        )
                        .into());
                    }
                };
            }
        }
        Ok(())
    }
}
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for &mut Level1
where
    E: std::convert::From<toql::error::ToqlError>,
    Level1Key: toql::from_row::FromRow<R, E>,
    Level2: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
{
    #[allow(unused_mut)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level1 as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
            self,
            descendents,
            field,
            rows,
            row_offset,
            index,
            selection_stream,
        )
    }
}

impl toql::toql_api::insert::Insert for Level2 where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::toql_api::insert::Insert for &mut Level2 where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::toql_api::update::Update for Level2 where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::toql_api::update::Update for &mut Level2 where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::toql_api::delete::Delete for Level2 where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap
{
}
impl toql::toql_api::delete::Delete for &Level2 where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap
{
}
impl<R, E> toql::toql_api::load::Load<R, E> for Level2
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl<R, E> toql::toql_api::load::Load<R, E> for &Level2
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl toql::toql_api::count::Count for Level2 where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped
{
}
impl toql::toql_api::count::Count for &Level2 where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped
{
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Level2Key {
    pub id: u64,
    pub level3: <Join<Level3> as toql::keyed::Keyed>::Key,
}
impl toql::key_fields::KeyFields for Level2Key {
    type Entity = Level2;
    fn fields() -> Vec<String> {
        let mut fields: Vec<String> = Vec::new();
        fields.push(String::from("id"));
        <<Join<Level3> as toql::keyed::Keyed>::Key as toql::key_fields::KeyFields>::fields()
            .iter()
            .for_each(|other_field| {
                fields.push(format!("{}_{}", "level3", other_field));
            });
        fields
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params.extend_from_slice(&toql::key::Key::params(&key.level3));
        params
    }
}
impl toql::key_fields::KeyFields for &Level2Key {
    type Entity = Level2;
    fn fields() -> Vec<String> {
        <Level2Key as toql::key_fields::KeyFields>::fields()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <Level2Key as toql::key_fields::KeyFields>::params(self)
    }
}
impl toql::key::Key for Level2Key {
    type Entity = Level2;
    fn columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("id"));
        <<Join<Level3> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
            .iter()
            .for_each(|other_column| {
                let default_self_column = format!("level3_{}", other_column);
                let column = {
                    if cfg!(debug_assertions) {
                        let valid_columns =
                            <<Level3 as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                        let invalid_columns: Vec<String> = ["id"]
                            .iter()
                            .filter(|col| !valid_columns.iter().any(|s| &s == col))
                            .map(|c| c.to_string())
                            .collect::<Vec<_>>();
                        if !invalid_columns.is_empty() {
                            toql::tracing::warn!(
                                "On `{}::{}` invalid columns found: `{}`. Valid columns are: `{}`",
                                "Level2",
                                "level3",
                                invalid_columns.join(","),
                                valid_columns.join(",")
                            );
                        }
                    }
                    let self_column = match other_column.as_str() {
                        "id" => "level3_id",
                        _ => &default_self_column,
                    };
                    self_column
                };
                columns.push(column.to_string());
            });
        columns
    }
    fn default_inverse_columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("level2_id"));
        <<Join<Level3> as toql::keyed::Keyed>::Key as toql::key::Key>::default_inverse_columns()
            .iter()
            .for_each(|other_column| {
                let default_self_column = format!("level3_{}", other_column);
                let column = {
                    if cfg!(debug_assertions) {
                        let valid_columns =
                            <<Level3 as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                        let invalid_columns: Vec<String> = ["id"]
                            .iter()
                            .filter(|col| !valid_columns.iter().any(|s| &s == col))
                            .map(|c| c.to_string())
                            .collect::<Vec<_>>();
                        if !invalid_columns.is_empty() {
                            toql::tracing::warn!(
                                "On `{}::{}` invalid columns found: `{}`. Valid columns are: `{}`",
                                "Level2",
                                "level3",
                                invalid_columns.join(","),
                                valid_columns.join(",")
                            );
                        }
                    }
                    let self_column = match other_column.as_str() {
                        "id" => "level3_id",
                        _ => &default_self_column,
                    };
                    self_column
                };
                columns.push(column.to_string());
            });
        columns
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params.extend_from_slice(&toql::key::Key::params(&key.level3));
        params
    }
}
impl toql::key::Key for &Level2Key {
    type Entity = Level2;
    fn columns() -> Vec<String> {
        <Level2Key as toql::key::Key>::columns()
    }
    fn default_inverse_columns() -> Vec<String> {
        <Level2Key as toql::key::Key>::default_inverse_columns()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <Level2Key as toql::key::Key>::params(self)
    }
}
impl From<Level2Key> for toql::sql_arg::SqlArg {
    fn from(t: Level2Key) -> toql::sql_arg::SqlArg {
        toql::sql_arg::SqlArg::from(t.id)
    }
}
impl From<&Level2Key> for toql::sql_arg::SqlArg {
    fn from(t: &Level2Key) -> toql::sql_arg::SqlArg {
        toql::sql_arg::SqlArg::from(t.id.to_owned())
    }
}
impl toql::keyed::Keyed for Level2 {
    type Key = Level2Key;
    fn key(&self) -> Self::Key {
        Level2Key {
            id: self.id.to_owned(),
            level3: <Join<Level3> as toql::keyed::Keyed>::key(&self.level3),
        }
    }
}
impl toql::keyed::Keyed for &Level2 {
    type Key = Level2Key;
    fn key(&self) -> Self::Key {
        <Level2 as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::Keyed for &mut Level2 {
    type Key = Level2Key;
    fn key(&self) -> Self::Key {
        <Level2 as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::KeyedMut for Level2 {
    fn set_key(&mut self, key: Self::Key) {
        self.id = key.id;
        <Join<Level3> as toql::keyed::KeyedMut>::set_key(&mut self.level3, key.level3);
    }
}
impl toql::keyed::KeyedMut for &mut Level2 {
    fn set_key(&mut self, key: Self::Key) {
        <Level2 as toql::keyed::KeyedMut>::set_key(self, key)
    }
}
impl std::convert::TryFrom<Vec<toql::sql_arg::SqlArg>> for Level2Key {
    type Error = toql::error::ToqlError;
    fn try_from(args: Vec<toql::sql_arg::SqlArg>) -> toql::result::Result<Self> {
        use std::convert::TryInto;
        Ok(Level2Key {
            id: args
                .get(0)
                .ok_or(toql::error::ToqlError::ValueMissing("id".to_string()))?
                .try_into()?,
            level3: <Join<Level3> as toql::keyed::Keyed>::Key::try_from(Vec::from(&args[1..]))?,
        })
    }
}
impl std::convert::From<(u64, <Level3 as toql::keyed::Keyed>::Key)> for Level2Key {
    fn from(key: (u64, <Level3 as toql::keyed::Keyed>::Key)) -> Self {
        Self {
            id: key.0,
            level3: key.1,
        }
    }
}
impl std::hash::Hash for Level2 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        <Level2 as toql::keyed::Keyed>::key(self).hash(state);
    }
}
impl toql::identity::Identity for Level2 {
    fn columns() -> Vec<String> {
        let mut columns = Vec::with_capacity(1usize);
        columns.push(String::from("id"));
        columns
    }
    fn set_column(
        &mut self,
        column: &str,
        value: &toql::sql_arg::SqlArg,
    ) -> toql::result::Result<()> {
        use std::convert::TryInto;
        match column {
            "id" => self.id = value.try_into()?,
            _ => {}
        }
        Ok(())
    }
}

impl toql::table_mapper::mapped::Mapped for Level2 {
    fn type_name() -> String {
        String::from("Level2")
    }
    fn table_name() -> String {
        String::from("Level2")
    }
    fn table_alias() -> String {
        String::from("level2")
    }
    #[allow(redundant_semicolons)]
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        mapper.map_column_with_options(
            "id",
            "id",
            toql::table_mapper::field_options::FieldOptions::new()
                .key(true)
                .preselect(true),
        );
        mapper .
        map_join_with_options("level3", "Level3", toql :: table_mapper ::
                              join_type :: JoinType :: Inner,
                              {
                                  let mut t = toql :: sql_expr :: SqlExpr ::
                                  literal(< Level3 as toql :: table_mapper ::
                                          mapped :: Mapped > :: table_name())
                                  ; t . push_literal(" ") ; t .
                                  push_other_alias() ; t
                              },
                              {
                                  let mut t = toql :: sql_expr :: SqlExpr ::
                                  new() ; [String :: from("id")] . iter() .
                                  for_each(| other_column |
                                           {
                                               let default_self_column =
                                               format !
                                               ("level3_{}", other_column) ; ;
                                               let self_column =
                                               {
                                                   if cfg ! (debug_assertions)
                                                   {
                                                       let valid_columns = <<
                                                       Level3 as toql :: keyed
                                                       :: Keyed > :: Key as
                                                       toql :: key :: Key > ::
                                                       columns() ; let
                                                       invalid_columns : Vec <
                                                       String > = ["id"] .
                                                       iter() .
                                                       filter(| col | !
                                                              valid_columns .
                                                              iter() .
                                                              any(| s | & s ==
                                                                  col)) .
                                                       map(| c | c .
                                                           to_string()) .
                                                       collect :: < Vec < _ >>
                                                       () ; if !
                                                       invalid_columns .
                                                       is_empty()
                                                       {
                                                           toql :: tracing ::
                                                           warn !
                                                           ("On `{}::{}` invalid columns found: `{}`. Valid columns are: `{}`",
                                                            "Level2",
                                                            "level3",
                                                            invalid_columns .
                                                            join(","),
                                                            valid_columns .
                                                            join(",")) ;
                                                       }
                                                   } let self_column = match
                                                   other_column . as_str()
                                                   {
                                                       "id" => "level3_id", _
                                                       => &
                                                       default_self_column
                                                   } ; self_column
                                               } ; t . push_self_alias() ; t .
                                               push_literal(".") ; t .
                                               push_literal(self_column) ; t .
                                               push_literal(" = ") ; t .
                                               push_other_alias() ; t .
                                               push_literal(".") ; t .
                                               push_literal(other_column) ; t
                                               . push_literal(" AND ") ;
                                           }) ; t . pop_literals(5) ; t
                              }, toql :: table_mapper :: join_options ::
                              JoinOptions :: new() . preselect(true) .
                              key(true)) ;
        mapper.map_column_with_options(
            "text",
            "text",
            toql::table_mapper::field_options::FieldOptions::new().preselect(true),
        );
        Ok(())
    }
}
impl toql::table_mapper::mapped::Mapped for &Level2 {
    fn type_name() -> String {
        <Level2 as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Level2 as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Level2 as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Level2 as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}
impl toql::table_mapper::mapped::Mapped for &mut Level2 {
    fn type_name() -> String {
        <Level2 as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Level2 as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Level2 as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Level2 as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Level2Key
where
    E: std::convert::From<toql::error::ToqlError>,
    u64: toql::from_row::FromRow<R, E>,
    Level3: toql::from_row::FromRow<R, E> + toql::keyed::Keyed,
    <Level3 as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(
            0 + <u64 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                + <Level3 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?,
        )
    }
    #[allow(unused_variables, unused_mut)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Level2Key>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        Ok(Some(Level2Key {
            id: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Level2Key::id".to_string(),
                    )
                    .into(),
                )?
            },
            level3: {
                <<Join<Level3> as toql::keyed::Keyed>::Key>::from_row(row, i, iter)?.ok_or(
                    toql::error::ToqlError::ValueMissing("Level2Key::level3".to_string()),
                )?
            },
        }))
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Level2
where
    E: std::convert::From<toql::error::ToqlError>,
    String: toql::from_row::FromRow<R, E>,
    u64: toql::from_row::FromRow<R, E>,
    Level3: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(
            0 + <u64 as toql::from_row::FromRow<_, E>>::forward(&mut iter)?
                + if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Level3 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                } else {
                    0
                }
                + <String as toql::from_row::FromRow<_, E>>::forward(&mut iter)?,
        )
    }
    #[allow(unused_variables, unused_mut, unused_imports)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Level2>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        use toql::sql_builder::select_stream::Select;
        Ok(Some(Level2 {
            id: {
                match <u64 as toql::from_row::FromRow<_, E>>::from_row(row, i, iter)? {
                    Some(s) => s,
                    _ => return Ok(None),
                }
            },
            level3: {
                if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    match toql::from_row::FromRow::<_, E>::from_row(row, i, iter)? {
                        Some(s) => s,
                        None => {
                            return Err(
                                toql::error::ToqlError::ValueMissing("level3".to_string()).into()
                            )
                        }
                    }
                } else {
                    return Err(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::SelectionExpected(
                            "level3".to_string(),
                        ),
                    )
                    .into());
                }
            },
            text: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Level2::text".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl toql::query_fields::QueryFields for Level2 {
    type FieldsType = Level2Fields;
    fn fields() -> Level2Fields {
        Level2Fields::new()
    }
    fn fields_from_path(path: String) -> Level2Fields {
        Level2Fields::from_path(path)
    }
}
pub struct Level2Fields(String);
impl toql::query_path::QueryPath for Level2Fields {
    fn into_path(self) -> String {
        self.0
    }
}
impl Level2Fields {
    pub fn new() -> Self {
        Self::from_path(String::from(""))
    }
    pub fn from_path(path: String) -> Self {
        Self(path)
    }
    pub fn into_name(self) -> String {
        self.0
    }
    pub fn id(mut self) -> toql::query::field::Field {
        self.0.push_str("id");
        toql::query::field::Field::from(self.0)
    }
    pub fn level3(mut self) -> <Level3 as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("level3_");
        <Level3 as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
    pub fn text(mut self) -> toql::query::field::Field {
        self.0.push_str("text");
        toql::query::field::Field::from(self.0)
    }
}

impl toql::tree::tree_insert::TreeInsert for Level2 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        let mut e = toql::sql_expr::SqlExpr::new();
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level3" => {
                    return Ok(<Level3 as toql::tree::tree_insert::TreeInsert>::columns(
                        descendents,
                    )?);
                }
                f @ _ => {
                    return Err(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        )
                        .into(),
                    );
                }
            },
            None => {
                e.push_literal("(");
                e.push_literal("id");
                e.push_literal(", ");
                for other_column in
                    <<Level3 as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("level3_{}", other_column);
                    let self_column = {
                        if cfg!(debug_assertions) {
                            let valid_columns =
                                <<Level3 as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                            let invalid_columns: Vec<String> = ["id"]
                                .iter()
                                .filter(|col| !valid_columns.iter().any(|s| &s == col))
                                .map(|c| c.to_string())
                                .collect::<Vec<_>>();
                            if !invalid_columns.is_empty() {
                                toql :: tracing :: warn !
                                ("On `{}::{}` invalid columns found: `{}`. Valid columns are: `{}`",
                                 "Level2", "level3", invalid_columns .
                                 join(","), valid_columns . join(","));
                            }
                        }
                        let self_column = match other_column.as_str() {
                            "id" => "level3_id",
                            _ => &default_self_column,
                        };
                        self_column
                    };
                    e.push_literal(self_column);
                    e.push_literal(", ");
                }
                e.push_literal("text");
                e.push_literal(", ");
                e.pop_literals(2);
                e.push_literal(")");
            }
        }
        Ok(e)
    }
    #[allow(unused_mut, unused_variables)]
    fn values<'a, 'b, I, J>(
        &self,
        mut descendents: I,
        roles: &std::collections::HashSet<String>,
        mut should_insert: &mut J,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
        J: Iterator<Item = &'b bool>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level3" => toql::tree::tree_insert::TreeInsert::values(
                    &self.level3,
                    descendents,
                    roles,
                    should_insert,
                    values,
                )?,
                f @ _ => {
                    return Err(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        )
                        .into(),
                    );
                }
            },
            None => {
                if !*should_insert.next().unwrap_or(&false) {
                    return Ok(());
                }
                values.push_literal("(");
                values.push_arg(toql::sql_arg::SqlArg::from(&self.id));
                values.push_literal(", ");
                &toql::key::Key::params(&toql::keyed::Keyed::key(&self.level3))
                    .into_iter()
                    .for_each(|a| {
                        values.push_arg(a);
                        values.push_literal(", ");
                    });
                values.push_arg(toql::sql_arg::SqlArg::from(&self.text));
                values.push_literal(", ");
                values.pop_literals(2);
                values.push_literal("), ");
            }
        }
        Ok(())
    }
}
impl toql::tree::tree_insert::TreeInsert for &Level2 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level2 as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, 'b, I, J>(
        &self,
        mut descendents: I,
        roles: &std::collections::HashSet<String>,
        mut should_insert: &mut J,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
        J: Iterator<Item = &'b bool>,
    {
        <Level2 as toql::tree::tree_insert::TreeInsert>::values(
            self,
            descendents,
            roles,
            should_insert,
            values,
        )
    }
}
impl toql::tree::tree_insert::TreeInsert for &mut Level2 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level2 as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, 'b, I, J>(
        &self,
        mut descendents: I,
        roles: &std::collections::HashSet<String>,
        mut should_insert: &mut J,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
        J: Iterator<Item = &'b bool>,
    {
        <Level2 as toql::tree::tree_insert::TreeInsert>::values(
            self,
            descendents,
            roles,
            should_insert,
            values,
        )
    }
}

impl toql::tree::tree_update::TreeUpdate for Level2 {
    #[allow(unused_mut, unused_variables, unused_parens)]
    fn update<'a, I>(
        &self,
        mut descendents: I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                f @ _ => {
                    return Err(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        )
                        .into(),
                    );
                }
            },
            None => {
                let key = <Self as toql::keyed::Keyed>::key(&self);
                if !toql::sql_arg::valid_key(&toql::key::Key::params(&key)) {
                    return Ok(());
                }
                let path_selected = fields.contains("*");
                let mut expr = toql::sql_expr::SqlExpr::new();
                expr.push_literal("UPDATE ");
                expr.push_literal("Level2");
                expr.push_literal(" SET ");
                let tokens = expr.tokens().len();
                if ((path_selected) || fields.contains("text")) {
                    expr.push_literal("text");
                    expr.push_literal(" = ");
                    expr.push_arg(toql::sql_arg::SqlArg::from(&self.text));
                    expr.push_literal(", ");
                }
                expr.pop();
                if expr.tokens().len() > tokens {
                    expr.push_literal(" WHERE ");
                    let key = <Self as toql::keyed::Keyed>::key(&self);
                    let resolver =
                        toql::sql_expr::resolver::Resolver::new().with_self_alias("Level2");
                    expr.extend(
                        resolver.alias_to_literals(&toql::key::Key::predicate_expr(&key, false))?,
                    );
                    exprs.push(expr);
                }
            }
        };
        Ok(())
    }
}
impl toql::tree::tree_update::TreeUpdate for &mut Level2 {
    #[allow(unused_mut)]
    fn update<'a, I>(
        &self,
        mut descendents: I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level2 as toql::tree::tree_update::TreeUpdate>::update(
            self,
            descendents,
            fields,
            roles,
            exprs,
        )
    }
}

impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for Level2
where
    E: std::convert::From<toql::error::ToqlError>,
    Level2Key: toql::from_row::FromRow<R, E>,
    Level3: toql::tree::tree_index::TreeIndex<R, E>,
{
    #[allow(unused_variables, unused_mut)]
    fn index<'a, I>(
        mut descendents: I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        use toql::from_row::FromRow;
        use toql::sql_builder::select_stream::Select;
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level3" => <Level3 as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    descendents,
                    rows,
                    row_offset,
                    index,
                )?,
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    )
                    .into());
                }
            },
            None => {
                for (n, row) in rows.into_iter().enumerate() {
                    let mut iter = std::iter::repeat(&Select::Query);
                    let mut i = row_offset;
                    let fk = Level2Key::from_row(&row, &mut i, &mut iter)?.ok_or(
                        toql::error::ToqlError::ValueMissing(
                            <Level2Key as toql::key::Key>::columns().join(", "),
                        ),
                    )?;
                    let mut s = DefaultHasher::new();
                    fk.hash(&mut s);
                    let fk_hash = s.finish();
                    index
                        .entry(fk_hash)
                        .and_modify(|h| h.push(n))
                        .or_insert(vec![n]);
                }
            }
        }
        Ok(())
    }
}
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for &Level2
where
    E: std::convert::From<toql::error::ToqlError>,
    Level2Key: toql::from_row::FromRow<R, E>,
    Level3: toql::tree::tree_index::TreeIndex<R, E>,
{
    #[allow(unused_mut)]
    fn index<'a, I>(
        mut descendents: I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level2 as toql::tree::tree_index::TreeIndex<R, E>>::index(
            descendents,
            rows,
            row_offset,
            index,
        )
    }
}

impl toql::tree::tree_identity::TreeIdentity for Level2 {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level3" => {
                    Ok(<Level3 as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)?)
                }
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    )
                    .into());
                }
            },
            None => Ok(false),
        }
    }
    #[allow(unused_variables, unused_mut)]
    fn set_id<'a, 'b, I>(
        &mut self,
        mut descendents: I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level3" => toql::tree::tree_identity::TreeIdentity::set_id(
                    &mut self.level3,
                    descendents,
                    action,
                )?,
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    )
                    .into());
                }
            },
            None => {
                fn set_self_key(
                    entity: &mut Level2,
                    args: &mut Vec<toql::sql_arg::SqlArg>,
                    invalid_only: bool,
                ) -> std::result::Result<(), toql::error::ToqlError> {
                    if invalid_only {
                        let self_key = toql::keyed::Keyed::key(&entity);
                        let self_key_params = toql::key::Key::params(&self_key);
                        if toql::sql_arg::valid_key(&self_key_params) {
                            return Ok(());
                        }
                    }
                    let n =
                        <<Level2 as toql::keyed::Keyed>::Key as toql::key::Key>::columns().len();
                    let end = args.len();
                    let args: Vec<toql::sql_arg::SqlArg> =
                        args.drain(end - n..).collect::<Vec<_>>();
                    let key = std::convert::TryFrom::try_from(args)?;
                    toql::keyed::KeyedMut::set_key(entity, key);
                    Ok(())
                }
                if let toql::tree::tree_identity::IdentityAction::SetInvalid(args) = action {
                    set_self_key(self, &mut args.borrow_mut(), true)?;
                }
                if let toql::tree::tree_identity::IdentityAction::Set(args) = action {
                    set_self_key(self, &mut args.borrow_mut(), false)?;
                }
                let self_key = toql::keyed::Keyed::key(&self);
                let self_key_params = toql::key::Key::params(&self_key);
                let self_key_columns =
                    <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
            }
        }
        Ok(())
    }
}
impl toql::tree::tree_identity::TreeIdentity for &mut Level2 {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level2 as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)
    }
    #[allow(unused_mut)]
    fn set_id<'a, 'b, I>(
        &mut self,
        mut descendents: I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        toql::tree::tree_identity::TreeIdentity::set_id(*self, descendents, action)
    }
}

impl toql::tree::tree_map::TreeMap for Level2 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        if registry.get("Level2").is_none() {
            registry.insert_new_mapper::<Level2>()?;
        }
        <Level3 as toql::tree::tree_map::TreeMap>::map(registry)?;
        Ok(())
    }
}
impl toql::tree::tree_map::TreeMap for &Level2 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        <Level2 as toql::tree::tree_map::TreeMap>::map(registry)
    }
}
impl toql::tree::tree_map::TreeMap for &mut Level2 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        <Level2 as toql::tree::tree_map::TreeMap>::map(registry)
    }
}

impl toql::tree::tree_predicate::TreePredicate for Level2 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        Ok(match descendents.next() {
            Some(d) => match d.as_str() {
                "level3" => {
                    <Level3 as toql::tree::tree_predicate::TreePredicate>::columns(descendents)?
                }
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    ));
                }
            },
            None => <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns(),
        })
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level3" => toql::tree::tree_predicate::TreePredicate::args(
                    &self.level3,
                    descendents,
                    args,
                )?,
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    ));
                }
            },
            None => {
                let key = <Self as toql::keyed::Keyed>::key(&self);
                args.extend(<<Self as toql::keyed::Keyed>::Key as toql::key::Key>::params(&key));
            }
        }
        Ok(())
    }
}
impl toql::tree::tree_predicate::TreePredicate for &Level2 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level2 as toql::tree::tree_predicate::TreePredicate>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level2 as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl toql::tree::tree_predicate::TreePredicate for &mut Level2 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level2 as toql::tree::tree_predicate::TreePredicate>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level2 as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}

impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for Level2
where
    E: std::convert::From<toql::error::ToqlError>,
    Level2Key: toql::from_row::FromRow<R, E>,
    Level3: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
{
    #[allow(unreachable_code, unused_variables, unused_mut, unused_imports)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        use toql::from_row::FromRow;
        use toql::keyed::Keyed;
        use toql::sql_builder::select_stream::Select;
        match descendents.next() {
            Some(d) => match d.as_str() {
                "level3" => toql::tree::tree_merge::TreeMerge::merge(
                    &mut self.level3,
                    descendents,
                    &field,
                    rows,
                    row_offset,
                    index,
                    selection_stream,
                )?,
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    )
                    .into());
                }
            },
            None => {
                let pk: Level2Key = <Self as toql::keyed::Keyed>::key(&self);
                let mut s = DefaultHasher::new();
                pk.hash(&mut s);
                let h = s.finish();
                let default_vec: Vec<usize> = Vec::new();
                let row_numbers: &Vec<usize> = index.get(&h).unwrap_or(&default_vec);
                let n = row_offset;
                match field {
                    f @ _ => {
                        return Err(toql::error::ToqlError::SqlBuilderError(
                            toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                                f.to_string(),
                            ),
                        )
                        .into());
                    }
                };
            }
        }
        Ok(())
    }
}
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for &mut Level2
where
    E: std::convert::From<toql::error::ToqlError>,
    Level2Key: toql::from_row::FromRow<R, E>,
    Level3: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
{
    #[allow(unused_mut)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level2 as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
            self,
            descendents,
            field,
            rows,
            row_offset,
            index,
            selection_stream,
        )
    }
}

impl toql::toql_api::insert::Insert for Level3 where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::toql_api::insert::Insert for &mut Level3 where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::toql_api::update::Update for Level3 where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::toql_api::update::Update for &mut Level3 where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::toql_api::delete::Delete for Level3 where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap
{
}
impl toql::toql_api::delete::Delete for &Level3 where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap
{
}
impl<R, E> toql::toql_api::load::Load<R, E> for Level3
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl<R, E> toql::toql_api::load::Load<R, E> for &Level3
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl toql::toql_api::count::Count for Level3 where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped
{
}
impl toql::toql_api::count::Count for &Level3 where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped
{
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Level3Key {
    pub id: u64,
}
impl toql::key_fields::KeyFields for Level3Key {
    type Entity = Level3;
    fn fields() -> Vec<String> {
        let mut fields: Vec<String> = Vec::new();
        fields.push(String::from("id"));
        fields
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params
    }
}
impl toql::key_fields::KeyFields for &Level3Key {
    type Entity = Level3;
    fn fields() -> Vec<String> {
        <Level3Key as toql::key_fields::KeyFields>::fields()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <Level3Key as toql::key_fields::KeyFields>::params(self)
    }
}
impl toql::key::Key for Level3Key {
    type Entity = Level3;
    fn columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("id"));
        columns
    }
    fn default_inverse_columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("level3_id"));
        columns
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params
    }
}
impl toql::key::Key for &Level3Key {
    type Entity = Level3;
    fn columns() -> Vec<String> {
        <Level3Key as toql::key::Key>::columns()
    }
    fn default_inverse_columns() -> Vec<String> {
        <Level3Key as toql::key::Key>::default_inverse_columns()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <Level3Key as toql::key::Key>::params(self)
    }
}
impl From<Level3Key> for toql::sql_arg::SqlArg {
    fn from(t: Level3Key) -> toql::sql_arg::SqlArg {
        toql::sql_arg::SqlArg::from(t.id)
    }
}
impl From<&Level3Key> for toql::sql_arg::SqlArg {
    fn from(t: &Level3Key) -> toql::sql_arg::SqlArg {
        toql::sql_arg::SqlArg::from(t.id.to_owned())
    }
}
impl toql::keyed::Keyed for Level3 {
    type Key = Level3Key;
    fn key(&self) -> Self::Key {
        Level3Key {
            id: self.id.to_owned(),
        }
    }
}
impl toql::keyed::Keyed for &Level3 {
    type Key = Level3Key;
    fn key(&self) -> Self::Key {
        <Level3 as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::Keyed for &mut Level3 {
    type Key = Level3Key;
    fn key(&self) -> Self::Key {
        <Level3 as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::KeyedMut for Level3 {
    fn set_key(&mut self, key: Self::Key) {
        self.id = key.id;
    }
}
impl toql::keyed::KeyedMut for &mut Level3 {
    fn set_key(&mut self, key: Self::Key) {
        <Level3 as toql::keyed::KeyedMut>::set_key(self, key)
    }
}
impl std::convert::TryFrom<Vec<toql::sql_arg::SqlArg>> for Level3Key {
    type Error = toql::error::ToqlError;
    fn try_from(args: Vec<toql::sql_arg::SqlArg>) -> toql::result::Result<Self> {
        use std::convert::TryInto;
        Ok(Level3Key {
            id: args
                .get(0)
                .ok_or(toql::error::ToqlError::ValueMissing("id".to_string()))?
                .try_into()?,
        })
    }
}
impl std::convert::From<u64> for Level3Key {
    fn from(key: u64) -> Self {
        Self { id: key }
    }
}
impl std::hash::Hash for Level3 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        <Level3 as toql::keyed::Keyed>::key(self).hash(state);
    }
}
impl toql::identity::Identity for Level3 {
    fn columns() -> Vec<String> {
        let mut columns = Vec::with_capacity(1usize);
        columns.push(String::from("id"));
        columns
    }
    fn set_column(
        &mut self,
        column: &str,
        value: &toql::sql_arg::SqlArg,
    ) -> toql::result::Result<()> {
        use std::convert::TryInto;
        match column {
            "id" => self.id = value.try_into()?,
            _ => {}
        }
        Ok(())
    }
}

impl toql::table_mapper::mapped::Mapped for Level3 {
    fn type_name() -> String {
        String::from("Level3")
    }
    fn table_name() -> String {
        String::from("Level3")
    }
    fn table_alias() -> String {
        String::from("level3")
    }
    #[allow(redundant_semicolons)]
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        mapper.map_column_with_options(
            "id",
            "id",
            toql::table_mapper::field_options::FieldOptions::new()
                .key(true)
                .preselect(true),
        );
        mapper.map_column_with_options(
            "text",
            "text",
            toql::table_mapper::field_options::FieldOptions::new().preselect(true),
        );
        Ok(())
    }
}
impl toql::table_mapper::mapped::Mapped for &Level3 {
    fn type_name() -> String {
        <Level3 as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Level3 as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Level3 as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Level3 as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}
impl toql::table_mapper::mapped::Mapped for &mut Level3 {
    fn type_name() -> String {
        <Level3 as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Level3 as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Level3 as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Level3 as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Level3Key
where
    E: std::convert::From<toql::error::ToqlError>,
    u64: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(0 + <u64 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?)
    }
    #[allow(unused_variables, unused_mut)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Level3Key>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        Ok(Some(Level3Key {
            id: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Level3Key::id".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Level3
where
    E: std::convert::From<toql::error::ToqlError>,
    u64: toql::from_row::FromRow<R, E>,
    String: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(
            0 + <u64 as toql::from_row::FromRow<_, E>>::forward(&mut iter)?
                + <String as toql::from_row::FromRow<_, E>>::forward(&mut iter)?,
        )
    }
    #[allow(unused_variables, unused_mut, unused_imports)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Level3>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        use toql::sql_builder::select_stream::Select;
        Ok(Some(Level3 {
            id: {
                match <u64 as toql::from_row::FromRow<_, E>>::from_row(row, i, iter)? {
                    Some(s) => s,
                    _ => return Ok(None),
                }
            },
            text: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Level3::text".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl toql::query_fields::QueryFields for Level3 {
    type FieldsType = Level3Fields;
    fn fields() -> Level3Fields {
        Level3Fields::new()
    }
    fn fields_from_path(path: String) -> Level3Fields {
        Level3Fields::from_path(path)
    }
}
pub struct Level3Fields(String);
impl toql::query_path::QueryPath for Level3Fields {
    fn into_path(self) -> String {
        self.0
    }
}
impl Level3Fields {
    pub fn new() -> Self {
        Self::from_path(String::from(""))
    }
    pub fn from_path(path: String) -> Self {
        Self(path)
    }
    pub fn into_name(self) -> String {
        self.0
    }
    pub fn id(mut self) -> toql::query::field::Field {
        self.0.push_str("id");
        toql::query::field::Field::from(self.0)
    }
    pub fn text(mut self) -> toql::query::field::Field {
        self.0.push_str("text");
        toql::query::field::Field::from(self.0)
    }
}

impl toql::tree::tree_insert::TreeInsert for Level3 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        let mut e = toql::sql_expr::SqlExpr::new();
        match descendents.next() {
            Some(d) => match d.as_str() {
                f @ _ => {
                    return Err(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        )
                        .into(),
                    );
                }
            },
            None => {
                e.push_literal("(");
                e.push_literal("id");
                e.push_literal(", ");
                e.push_literal("text");
                e.push_literal(", ");
                e.pop_literals(2);
                e.push_literal(")");
            }
        }
        Ok(e)
    }
    #[allow(unused_mut, unused_variables)]
    fn values<'a, 'b, I, J>(
        &self,
        mut descendents: I,
        roles: &std::collections::HashSet<String>,
        mut should_insert: &mut J,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
        J: Iterator<Item = &'b bool>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                f @ _ => {
                    return Err(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        )
                        .into(),
                    );
                }
            },
            None => {
                if !*should_insert.next().unwrap_or(&false) {
                    return Ok(());
                }
                values.push_literal("(");
                values.push_arg(toql::sql_arg::SqlArg::from(&self.id));
                values.push_literal(", ");
                values.push_arg(toql::sql_arg::SqlArg::from(&self.text));
                values.push_literal(", ");
                values.pop_literals(2);
                values.push_literal("), ");
            }
        }
        Ok(())
    }
}
impl toql::tree::tree_insert::TreeInsert for &Level3 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level3 as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, 'b, I, J>(
        &self,
        mut descendents: I,
        roles: &std::collections::HashSet<String>,
        mut should_insert: &mut J,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
        J: Iterator<Item = &'b bool>,
    {
        <Level3 as toql::tree::tree_insert::TreeInsert>::values(
            self,
            descendents,
            roles,
            should_insert,
            values,
        )
    }
}
impl toql::tree::tree_insert::TreeInsert for &mut Level3 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level3 as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, 'b, I, J>(
        &self,
        mut descendents: I,
        roles: &std::collections::HashSet<String>,
        mut should_insert: &mut J,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
        J: Iterator<Item = &'b bool>,
    {
        <Level3 as toql::tree::tree_insert::TreeInsert>::values(
            self,
            descendents,
            roles,
            should_insert,
            values,
        )
    }
}

impl toql::tree::tree_update::TreeUpdate for Level3 {
    #[allow(unused_mut, unused_variables, unused_parens)]
    fn update<'a, I>(
        &self,
        mut descendents: I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                f @ _ => {
                    return Err(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        )
                        .into(),
                    );
                }
            },
            None => {
                let key = <Self as toql::keyed::Keyed>::key(&self);
                if !toql::sql_arg::valid_key(&toql::key::Key::params(&key)) {
                    return Ok(());
                }
                let path_selected = fields.contains("*");
                let mut expr = toql::sql_expr::SqlExpr::new();
                expr.push_literal("UPDATE ");
                expr.push_literal("Level3");
                expr.push_literal(" SET ");
                let tokens = expr.tokens().len();
                if ((path_selected) || fields.contains("text")) {
                    expr.push_literal("text");
                    expr.push_literal(" = ");
                    expr.push_arg(toql::sql_arg::SqlArg::from(&self.text));
                    expr.push_literal(", ");
                }
                expr.pop();
                if expr.tokens().len() > tokens {
                    expr.push_literal(" WHERE ");
                    let key = <Self as toql::keyed::Keyed>::key(&self);
                    let resolver =
                        toql::sql_expr::resolver::Resolver::new().with_self_alias("Level3");
                    expr.extend(
                        resolver.alias_to_literals(&toql::key::Key::predicate_expr(&key, false))?,
                    );
                    exprs.push(expr);
                }
            }
        };
        Ok(())
    }
}
impl toql::tree::tree_update::TreeUpdate for &mut Level3 {
    #[allow(unused_mut)]
    fn update<'a, I>(
        &self,
        mut descendents: I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level3 as toql::tree::tree_update::TreeUpdate>::update(
            self,
            descendents,
            fields,
            roles,
            exprs,
        )
    }
}

impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for Level3
where
    E: std::convert::From<toql::error::ToqlError>,
    Level3Key: toql::from_row::FromRow<R, E>,
{
    #[allow(unused_variables, unused_mut)]
    fn index<'a, I>(
        mut descendents: I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        use toql::from_row::FromRow;
        use toql::sql_builder::select_stream::Select;
        match descendents.next() {
            Some(d) => match d.as_str() {
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    )
                    .into());
                }
            },
            None => {
                for (n, row) in rows.into_iter().enumerate() {
                    let mut iter = std::iter::repeat(&Select::Query);
                    let mut i = row_offset;
                    let fk = Level3Key::from_row(&row, &mut i, &mut iter)?.ok_or(
                        toql::error::ToqlError::ValueMissing(
                            <Level3Key as toql::key::Key>::columns().join(", "),
                        ),
                    )?;
                    let mut s = DefaultHasher::new();
                    fk.hash(&mut s);
                    let fk_hash = s.finish();
                    index
                        .entry(fk_hash)
                        .and_modify(|h| h.push(n))
                        .or_insert(vec![n]);
                }
            }
        }
        Ok(())
    }
}
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for &Level3
where
    E: std::convert::From<toql::error::ToqlError>,
    Level3Key: toql::from_row::FromRow<R, E>,
{
    #[allow(unused_mut)]
    fn index<'a, I>(
        mut descendents: I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level3 as toql::tree::tree_index::TreeIndex<R, E>>::index(
            descendents,
            rows,
            row_offset,
            index,
        )
    }
}

impl toql::tree::tree_identity::TreeIdentity for Level3 {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    )
                    .into());
                }
            },
            None => Ok(false),
        }
    }
    #[allow(unused_variables, unused_mut)]
    fn set_id<'a, 'b, I>(
        &mut self,
        mut descendents: I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    )
                    .into());
                }
            },
            None => {
                fn set_self_key(
                    entity: &mut Level3,
                    args: &mut Vec<toql::sql_arg::SqlArg>,
                    invalid_only: bool,
                ) -> std::result::Result<(), toql::error::ToqlError> {
                    if invalid_only {
                        let self_key = toql::keyed::Keyed::key(&entity);
                        let self_key_params = toql::key::Key::params(&self_key);
                        if toql::sql_arg::valid_key(&self_key_params) {
                            return Ok(());
                        }
                    }
                    let n =
                        <<Level3 as toql::keyed::Keyed>::Key as toql::key::Key>::columns().len();
                    let end = args.len();
                    let args: Vec<toql::sql_arg::SqlArg> =
                        args.drain(end - n..).collect::<Vec<_>>();
                    let key = std::convert::TryFrom::try_from(args)?;
                    toql::keyed::KeyedMut::set_key(entity, key);
                    Ok(())
                }
                if let toql::tree::tree_identity::IdentityAction::SetInvalid(args) = action {
                    set_self_key(self, &mut args.borrow_mut(), true)?;
                }
                if let toql::tree::tree_identity::IdentityAction::Set(args) = action {
                    set_self_key(self, &mut args.borrow_mut(), false)?;
                }
                let self_key = toql::keyed::Keyed::key(&self);
                let self_key_params = toql::key::Key::params(&self_key);
                let self_key_columns =
                    <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
            }
        }
        Ok(())
    }
}
impl toql::tree::tree_identity::TreeIdentity for &mut Level3 {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level3 as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)
    }
    #[allow(unused_mut)]
    fn set_id<'a, 'b, I>(
        &mut self,
        mut descendents: I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        toql::tree::tree_identity::TreeIdentity::set_id(*self, descendents, action)
    }
}

impl toql::tree::tree_map::TreeMap for Level3 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        if registry.get("Level3").is_none() {
            registry.insert_new_mapper::<Level3>()?;
        }
        Ok(())
    }
}
impl toql::tree::tree_map::TreeMap for &Level3 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        <Level3 as toql::tree::tree_map::TreeMap>::map(registry)
    }
}
impl toql::tree::tree_map::TreeMap for &mut Level3 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        <Level3 as toql::tree::tree_map::TreeMap>::map(registry)
    }
}

impl toql::tree::tree_predicate::TreePredicate for Level3 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        Ok(match descendents.next() {
            Some(d) => match d.as_str() {
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    ));
                }
            },
            None => <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns(),
        })
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    ));
                }
            },
            None => {
                let key = <Self as toql::keyed::Keyed>::key(&self);
                args.extend(<<Self as toql::keyed::Keyed>::Key as toql::key::Key>::params(&key));
            }
        }
        Ok(())
    }
}
impl toql::tree::tree_predicate::TreePredicate for &Level3 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level3 as toql::tree::tree_predicate::TreePredicate>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level3 as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl toql::tree::tree_predicate::TreePredicate for &mut Level3 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level3 as toql::tree::tree_predicate::TreePredicate>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level3 as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}

impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for Level3
where
    E: std::convert::From<toql::error::ToqlError>,
    Level3Key: toql::from_row::FromRow<R, E>,
{
    #[allow(unreachable_code, unused_variables, unused_mut, unused_imports)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        use toql::from_row::FromRow;
        use toql::keyed::Keyed;
        use toql::sql_builder::select_stream::Select;
        match descendents.next() {
            Some(d) => match d.as_str() {
                f @ _ => {
                    return Err(toql::error::ToqlError::SqlBuilderError(
                        toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                            f.to_string(),
                        ),
                    )
                    .into());
                }
            },
            None => {
                let pk: Level3Key = <Self as toql::keyed::Keyed>::key(&self);
                let mut s = DefaultHasher::new();
                pk.hash(&mut s);
                let h = s.finish();
                let default_vec: Vec<usize> = Vec::new();
                let row_numbers: &Vec<usize> = index.get(&h).unwrap_or(&default_vec);
                let n = row_offset;
                match field {
                    f @ _ => {
                        return Err(toql::error::ToqlError::SqlBuilderError(
                            toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                                f.to_string(),
                            ),
                        )
                        .into());
                    }
                };
            }
        }
        Ok(())
    }
}
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for &mut Level3
where
    E: std::convert::From<toql::error::ToqlError>,
    Level3Key: toql::from_row::FromRow<R, E>,
{
    #[allow(unused_mut)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>> + Clone,
    {
        <Level3 as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
            self,
            descendents,
            field,
            rows,
            row_offset,
            index,
            selection_stream,
        )
    }
}
