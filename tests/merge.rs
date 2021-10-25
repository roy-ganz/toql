use pretty_assertions::assert_eq;
use toql::prelude::{fields, paths, query, Cache, Join, Result, SqlArg, Toql, ToqlApi};
use toql::mock_db::{row::Row, MockDb};
use toql::row;
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(merge())] // Default mapping Level1.id = Level2.level1_id
    level2: Vec<Level2> // Preselected merge
}
#[derive(Debug, Default, Toql)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    #[toql(key)]
    level1_id: u64,
    text: String,

    #[toql(merge(columns(self="id", other ="level2_id")))] // Specified columns
    level3: Option<Vec<Level3>> // Selectable merge
}

#[derive(Debug, Default, Toql)]
pub struct Level3 {
    #[toql(key)]
    id: u64,
    #[toql(key)]
    level2_id: u64,
    text: String,

    #[toql(merge( columns(self="id", other="level3_id"), join_sql="...text = 'ABC"))]    // Custom ON statement
    level4: Vec<Level4> // Preselected merge join
}
#[derive(Debug, Default, Toql)]
pub struct Level4 {
    #[toql(key)]
    id: u64,
    #[toql(key)]
    level3_id:u64,
    text: String,

} 
/* 
#[derive(Debug, Default)]
pub struct Level1 {
    id: u64,
    text: String,
    level2: Vec<Level2>, // Preselected merge
}
#[derive(Debug, Default)]
pub struct Level2 {
    id: u64,
    level1_id: u64,
    text: String,
    level3: Option<Vec<Level3>>, // Selectable merge
}

#[derive(Debug, Default)]
pub struct Level3 {
    id: u64,
    level2_id: u64,
    text: String,
    level4: Vec<Level4>, // Preselected merge join
}
#[derive(Debug, Default)]
pub struct Level4 {
    id: u64,

    level3_id: u64,
    text: String,
} */

fn populated_level() -> Level1 {
    let l4 = Level4 {
        id: 4,
        text: "level4".to_string(),
        level3_id: 3,
    };
    let l3 = Level3 {
        id: 3,
        text: "level3".to_string(),
        level2_id: 2,
        level4: vec![l4],
    };
    let l2 = Level2 {
        id: 2,
        text: "level2".to_string(),
        level1_id: 1,
        level3: Some(vec![l3]),
    };

    Level1 {
        id: 1,
        text: "level1".to_string(),
        level2: vec![l2],
    }
}

#[tokio::test]
#[traced_test("info")]
async fn load1() -> Result<()> {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load level 1 + preselected level 2
    let q = query!(Level1, "*");
    let select1 = "SELECT level1.id, level1.text FROM Level1 level1";
    let select2 = "SELECT level1.id, level2.id, level2.level1_id, level2.text \
                        FROM Level2 level2 \
                        JOIN Level1 level1 \
                        ON (level1.id = level2.level1_id AND level1.id = 1)";

    toql.mock_rows(select1, vec![row!(1u64, "level1")]);
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64, "level2")]);

    let r = toql.load_many(q).await?;

    assert_eq!(toql.take_unsafe_sqls(), [select1, select2]);

    Ok(())
}
#[tokio::test]
#[traced_test("info")]
async fn load2() -> Result<()> {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);
    
    // Load preselects from level 1..3 + all fields from level 4
    let q = query!(Level1, "*, level2_level3_level4_*");
    let select1 = "SELECT level1.id, level1.text FROM Level1 level1";
    let select2 = "SELECT level1.id, level2.id, level2.level1_id, level2.text \
                        FROM Level2 level2 \
                        JOIN Level1 level1 \
                        ON (level1.id = level2.level1_id \
                            AND level1.id = 1)";

    let select3 = "SELECT level1_level2.id, level1_level2.level1_id, level3.id, level3.level2_id, level3.text \
                        FROM Level3 level3 \
                        JOIN Level2 level1_level2 \
                        ON (level1_level2.id = level3.level2_id \
                            AND level1_level2.id = 2 \
                            AND level1_level2.level1_id = 1)";

    let select4 = "SELECT level1_level2_level3.id, level1_level2_level3.level2_id, level4.id, level4.level3_id, level4.text \
                        FROM Level4 level4 level4.text = \'ABC \
                        JOIN Level3 level1_level2_level3 \
                        ON (level1_level2_level3.id = level4.level3_id \
                            AND level1_level2_level3.id = ? \
                            AND level1_level2_level3.level2_id = ?)";

    // level1.id
    toql.mock_rows(select1, vec![row!(1u64, "level1")]); 

    // level1.id, level2.id, level2.level1_id
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64, "level2")]);        

    // level1_level2.id, level1_level2.level1_id, level3.id, level3.level2_id, level3.text
    toql.mock_rows(select3, vec![row!(2u64, 1u64, 3u64, 2u64, "level3")]);  

    // SELECT level1_level2_level3.id, level1_level2_level3.level2_id, level4.id, level4.level3_id, level4.text
    toql.mock_rows(select4, vec![row!(3u64, 2u64, 4u64, 3u64, "level4")]);

    let r = toql.load_many(q).await?;

    assert_eq!(
        toql.take_unsafe_sqls(),
        [select1, select2, select3, select4]
    );

    Ok(())
}

// insert 


// update




/* 
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

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Level1Key {
    pub id: u64,
}
impl toql::key_fields::KeyFields for Level1Key {
    type Entity = Level1;
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
        columns
    }
    fn default_inverse_columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("level1_id"));
        columns
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
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
        })
    }
}
impl std::convert::From<u64> for Level1Key {
    fn from(key: u64) -> Self {
        Self { id: key }
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
                "level2" => {
                    for f in &mut self.level2 {
                        <Level2 as toql::tree::tree_identity::TreeIdentity>::set_id(
                            f,
                            descendents.clone(),
                            action.clone(),
                        )?
                    }
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
                let self_key = <Self as toql::keyed::Keyed>::key(&self);
                let self_key_params = toql::key::Key::params(&self_key);
                let self_key_columns =
                    <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                for e in &mut self.level2 {
                    let key = <Level2 as toql::keyed::Keyed>::key(e);
                    let merge_key = toql::keyed::Keyed::key(e);
                    let merge_key_params = toql::key::Key::params(&merge_key);
                    let valid = toql::sql_arg::valid_key(&merge_key_params);
                    if matches!(
                        action,
                        toql::tree::tree_identity::IdentityAction::RefreshInvalid
                    ) && valid
                    {
                        continue;
                    }
                    if matches!(
                        action,
                        toql::tree::tree_identity::IdentityAction::RefreshValid
                    ) && !valid
                    {
                        continue;
                    }
                    for (self_key_column, self_key_param) in
                        self_key_columns.iter().zip(&self_key_params)
                    {
                        let calculated_other_column = match self_key_column.as_str() {
                            "id" => "level1_id",
                            x @ _ => x,
                        };
                        if cfg!(debug_assertions) {
                            let foreign_identity_columns =
                                <Level2 as toql::identity::Identity>::columns();
                            if !foreign_identity_columns
                                .contains(&calculated_other_column.to_string())
                            {
                                toql :: tracing :: warn !
                                ("`{}` cannot find column `{}` in `{}`. \
                                                            Try adding `#[toql(foreign_key)]` in `{}` to the missing field.",
                                 "Level1", calculated_other_column, "Level2",
                                 "Level2")
                            }
                        }
                        toql::identity::Identity::set_column(
                            e,
                            calculated_other_column,
                            self_key_param,
                        )?;
                    }
                }
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
        <Level1 as toql::tree::tree_identity::TreeIdentity>::set_id(self, descendents, action)
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
                "level2" => {
                    for f in &self.level2 {
                        <Level2 as toql::tree::tree_predicate::TreePredicate>::args(
                            f,
                            descendents.clone(),
                            args,
                        )?
                    }
                }
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
                    println!("Adding row {} for key {:?}", n, &fk);
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
                "level2" => {
                    for f in &mut self.level2 {
                        <Level2 as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                            f,
                            descendents.clone(),
                            &field,
                            rows,
                            row_offset,
                            index,
                            selection_stream,
                        )?
                    }
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
            None => {
                let pk: Level1Key = <Self as toql::keyed::Keyed>::key(&self);
                
                let mut s = DefaultHasher::new();
                pk.hash(&mut s);
                let h = s.finish();
                let default_vec: Vec<usize> = Vec::new();
                let row_numbers: &Vec<usize> = index.get(&h).unwrap_or(&default_vec);
                println!("Found rows {:?} for primary key {:?}", &row_numbers, &pk);
                let n = row_offset;
                match field {
                    "level2" => {
                        for row_number in row_numbers {
                            let mut i = n;
                            let mut iter = std::iter::repeat(&Select::Query);
                            let row: &R = &rows[*row_number];
                            let fk = Level1Key::from_row(&row, &mut i, &mut iter)?.ok_or(
                                toql::error::ToqlError::ValueMissing("level2".to_string()),
                            )?;
                            if fk == pk {
                                let mut iter = selection_stream.iter();
                                let e = Level2::from_row(&row, &mut i, &mut iter)?.ok_or(
                                    toql::error::ToqlError::ValueMissing("level2".to_string()),
                                )?;
                                self.level2.push(e);
                            }
                        }
                    }
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

impl<R, E> toql::from_row::FromRow<R, E> for Level1Key
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
        }))
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Level1
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
            text: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Level1::text".to_string(),
                    )
                    .into(),
                )?
            },
            level2: Vec::new(),
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
    pub fn text(mut self) -> toql::query::field::Field {
        self.0.push_str("text");
        toql::query::field::Field::from(self.0)
    }
    pub fn level2(mut self) -> <Level2 as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("level2_");
        <Level2 as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
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
        mapper.map_column_with_options(
            "text",
            "text",
            toql::table_mapper::field_options::FieldOptions::new().preselect(true),
        );
        mapper.map_merge_with_options(
            "level2",
            "Level2",
            {
                toql::sql_expr::SqlExpr::from(vec![
                    toql::sql_expr::SqlExprToken::Literal("JOIN ".to_string()),
                    toql::sql_expr::SqlExprToken::Literal("Level1".to_string()),
                    toql::sql_expr::SqlExprToken::Literal(" ".to_string()),
                    toql::sql_expr::SqlExprToken::SelfAlias,
                ])
            },
            {
                {
                    let mut tokens: Vec<toql::sql_expr::SqlExprToken> = Vec::new();
                    <Level1Key as toql::key::Key>::columns()
                        .iter()
                        .zip(<Level1Key as toql::key::Key>::default_inverse_columns())
                        .for_each(|(t, o)| {
                            tokens.extend(
                                vec![
                                    toql::sql_expr::SqlExprToken::SelfAlias,
                                    toql::sql_expr::SqlExprToken::Literal(".".to_string()),
                                    toql::sql_expr::SqlExprToken::Literal(t.to_string()),
                                    toql::sql_expr::SqlExprToken::Literal(" = ".to_string()),
                                    toql::sql_expr::SqlExprToken::OtherAlias,
                                    toql::sql_expr::SqlExprToken::Literal(".".to_string()),
                                    toql::sql_expr::SqlExprToken::Literal(o.to_string()),
                                    toql::sql_expr::SqlExprToken::Literal(" AND ".to_string()),
                                ]
                                .into_iter(),
                            )
                        });
                    tokens.pop();
                    toql::sql_expr::SqlExpr::from(tokens)
                }
            },
            toql::table_mapper::merge_options::MergeOptions::new().preselect(true),
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
                "level2" => {
                    for f in &self.level2 {
                        <Level2 as toql::tree::tree_insert::TreeInsert>::values(
                            f,
                            descendents.clone(),
                            roles,
                            should_insert,
                            values,
                        )?
                    }
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
                "level2" => {
                    for f in &self.level2 {
                        <Level2 as toql::tree::tree_update::TreeUpdate>::update(
                            f,
                            descendents.clone(),
                            fields,
                            roles,
                            exprs,
                        )?
                    }
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
                if (path_selected || fields.contains("text")) {
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

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Level2Key {
    pub id: u64,
    pub level1_id: u64,
}
impl toql::key_fields::KeyFields for Level2Key {
    type Entity = Level2;
    fn fields() -> Vec<String> {
        let mut fields: Vec<String> = Vec::new();
        fields.push(String::from("id"));
        fields.push(String::from("level1Id"));
        fields
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params.push(toql::sql_arg::SqlArg::from(&key.level1_id));
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
        columns.push(String::from("level1_id"));
        columns
    }
    fn default_inverse_columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("level2_id"));
        columns.push(String::from("level2_level1_id"));
        columns
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params.push(toql::sql_arg::SqlArg::from(&key.level1_id));
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
impl toql::keyed::Keyed for Level2 {
    type Key = Level2Key;
    fn key(&self) -> Self::Key {
        Level2Key {
            id: self.id.to_owned(),
            level1_id: self.level1_id.to_owned(),
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
        self.level1_id = key.level1_id;
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
            level1_id: args
                .get(1)
                .ok_or(toql::error::ToqlError::ValueMissing(
                    "level1_id".to_string(),
                ))?
                .try_into()?,
        })
    }
}
impl std::convert::From<(u64, u64)> for Level2Key {
    fn from(key: (u64, u64)) -> Self {
        Self {
            id: key.0,
            level1_id: key.1,
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
        let mut columns = Vec::with_capacity(2usize);
        columns.push(String::from("id"));
        columns.push(String::from("level1_id"));
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
            "level1_id" => self.level1_id = value.try_into()?,
            _ => {}
        }
        Ok(())
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
                "level3" => {
                    for f in self
                        .level3
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("level3".to_string()))?
                    {
                        <Level3 as toql::tree::tree_identity::TreeIdentity>::set_id(
                            f,
                            descendents.clone(),
                            action.clone(),
                        )?
                    }
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
                let self_key = <Self as toql::keyed::Keyed>::key(&self);
                let self_key_params = toql::key::Key::params(&self_key);
                let self_key_columns =
                    <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                if let Some(u) = self.level3.as_mut() {
                    for e in u {
                        let merge_key = toql::keyed::Keyed::key(e);
                        let merge_key_params = toql::key::Key::params(&merge_key);
                        let valid = toql::sql_arg::valid_key(&merge_key_params);
                        if matches!(
                            action,
                            toql::tree::tree_identity::IdentityAction::RefreshInvalid
                        ) && valid
                        {
                            continue;
                        }
                        if matches!(
                            action,
                            toql::tree::tree_identity::IdentityAction::RefreshValid
                        ) && !valid
                        {
                            continue;
                        }
                        for (self_key_column, self_key_param) in
                            self_key_columns.iter().zip(&self_key_params)
                        {
                            let calculated_other_column = match self_key_column.as_str() {
                                "id" => "level2_id",
                                x @ _ => x,
                            };
                            if cfg!(debug_assertions) {
                                let foreign_identity_columns =
                                    <Level3 as toql::identity::Identity>::columns();
                                if !foreign_identity_columns
                                    .contains(&calculated_other_column.to_string())
                                {
                                    toql :: tracing :: warn !
                                    ("`{}` cannot find column `{}` in `{}`. \
                                                            Try adding `#[toql(foreign_key)]` in `{}` to the missing field.",
                                     "Level2", calculated_other_column,
                                     "Level3", "Level3")
                                }
                            }
                            toql::identity::Identity::set_column(
                                e,
                                calculated_other_column,
                                self_key_param,
                            )?;
                        }
                    }
                }
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
        <Level2 as toql::tree::tree_identity::TreeIdentity>::set_id(self, descendents, action)
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
                "level3" => {
                    if let Some(ref fs) = self.level3 {
                        for f in fs {
                            <Level3 as toql::tree::tree_predicate::TreePredicate>::args(
                                f,
                                descendents.clone(),
                                args,
                            )?
                        }
                    }
                }
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
                     println!("Adding row {} for key {:?}", n, &fk);
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
                "level3" => {
                    for f in self
                        .level3
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("level3".to_string()))?
                    {
                        <Level3 as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                            f,
                            descendents.clone(),
                            &field,
                            rows,
                            row_offset,
                            index,
                            selection_stream,
                        )?
                    }
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
            None => {
                let pk: Level2Key = <Self as toql::keyed::Keyed>::key(&self);
                let mut s = DefaultHasher::new();
                pk.hash(&mut s);
                let h = s.finish();
                let default_vec: Vec<usize> = Vec::new();
                let row_numbers: &Vec<usize> = index.get(&h).unwrap_or(&default_vec);
                let n = row_offset;
                println!("Found rows {:?} for primary key {:?}", &row_numbers, &pk);
                match field {
                    "level3" => {
                        if self.level3.is_none() {
                            self.level3 = Some(Vec::new())
                        }
                        for row_number in row_numbers {
                            let mut i = n;
                            let mut iter = std::iter::repeat(&Select::Query);
                            let row: &R = &rows[*row_number];
                            let fk = Level2Key::from_row(&row, &mut i, &mut iter)?.ok_or(
                                toql::error::ToqlError::ValueMissing("level3".to_string()),
                            )?;
                            if fk == pk {
                                let mut iter = selection_stream.iter();
                                let e = Level3::from_row(&row, &mut i, &mut iter)?.ok_or(
                                    toql::error::ToqlError::ValueMissing("level3".to_string()),
                                )?;
                                self.level3.as_mut().unwrap().push(e);
                            }
                        }
                    }
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

impl<R, E> toql::from_row::FromRow<R, E> for Level2Key
where
    E: std::convert::From<toql::error::ToqlError>,
    u64: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(
            0 + <u64 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                + <u64 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?,
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
            level1_id: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Level2Key::level1_id".to_string(),
                    )
                    .into(),
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
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(
            0 + <u64 as toql::from_row::FromRow<_, E>>::forward(&mut iter)?
                + <u64 as toql::from_row::FromRow<_, E>>::forward(&mut iter)?
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
            level1_id: {
                match <u64 as toql::from_row::FromRow<_, E>>::from_row(row, i, iter)? {
                    Some(s) => s,
                    _ => return Ok(None),
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
            level3: None,
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
    pub fn level1_id(mut self) -> toql::query::field::Field {
        self.0.push_str("level1Id");
        toql::query::field::Field::from(self.0)
    }
    pub fn text(mut self) -> toql::query::field::Field {
        self.0.push_str("text");
        toql::query::field::Field::from(self.0)
    }
    pub fn level3(mut self) -> <Level3 as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("level3_");
        <Level3 as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
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
        mapper.map_column_with_options(
            "level1Id",
            "level1_id",
            toql::table_mapper::field_options::FieldOptions::new()
                .key(true)
                .preselect(true),
        );
        mapper.map_column_with_options(
            "text",
            "text",
            toql::table_mapper::field_options::FieldOptions::new().preselect(true),
        );
        mapper.map_merge_with_options(
            "level3",
            "Level3",
            {
                toql::sql_expr::SqlExpr::from(vec![
                    toql::sql_expr::SqlExprToken::Literal("JOIN ".to_string()),
                    toql::sql_expr::SqlExprToken::Literal("Level2".to_string()),
                    toql::sql_expr::SqlExprToken::Literal(" ".to_string()),
                    toql::sql_expr::SqlExprToken::SelfAlias,
                ])
            },
            {
                let mut t = toql::sql_expr::SqlExpr::new();
                t.push_self_alias();
                t.push_literal(".");
                t.push_literal("id");
                t.push_literal(" = ");
                t.push_other_alias();
                t.push_literal(".");
                t.push_literal("level2_id");
                t
            },
            toql::table_mapper::merge_options::MergeOptions::new(),
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
                e.push_literal("level1_id");
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
                "level3" => {
                    if let Some(fs) = self.level3.as_ref() {
                        for f in fs {
                            <Level3 as toql::tree::tree_insert::TreeInsert>::values(
                                f,
                                descendents.clone(),
                                roles,
                                should_insert,
                                values,
                            )?
                        }
                    }
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
                if !*should_insert.next().unwrap_or(&false) {
                    return Ok(());
                }
                values.push_literal("(");
                values.push_arg(toql::sql_arg::SqlArg::from(&self.id));
                values.push_literal(", ");
                values.push_arg(toql::sql_arg::SqlArg::from(&self.level1_id));
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
                "level3" => {
                    if let Some(fs) = self.level3.as_ref() {
                        for f in fs {
                            <Level3 as toql::tree::tree_update::TreeUpdate>::update(
                                f,
                                descendents.clone(),
                                fields,
                                roles,
                                exprs,
                            )?
                        }
                    }
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
                if (path_selected || fields.contains("text")) {
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

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Level3Key {
    pub id: u64,
    pub level2_id: u64,
}
impl toql::key_fields::KeyFields for Level3Key {
    type Entity = Level3;
    fn fields() -> Vec<String> {
        let mut fields: Vec<String> = Vec::new();
        fields.push(String::from("id"));
        fields.push(String::from("level2Id"));
        fields
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params.push(toql::sql_arg::SqlArg::from(&key.level2_id));
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
        columns.push(String::from("level2_id"));
        columns
    }
    fn default_inverse_columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("level3_id"));
        columns.push(String::from("level3_level2_id"));
        columns
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params.push(toql::sql_arg::SqlArg::from(&key.level2_id));
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
impl toql::keyed::Keyed for Level3 {
    type Key = Level3Key;
    fn key(&self) -> Self::Key {
        Level3Key {
            id: self.id.to_owned(),
            level2_id: self.level2_id.to_owned(),
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
        self.level2_id = key.level2_id;
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
            level2_id: args
                .get(1)
                .ok_or(toql::error::ToqlError::ValueMissing(
                    "level2_id".to_string(),
                ))?
                .try_into()?,
        })
    }
}
impl std::convert::From<(u64, u64)> for Level3Key {
    fn from(key: (u64, u64)) -> Self {
        Self {
            id: key.0,
            level2_id: key.1,
        }
    }
}
impl std::hash::Hash for Level3 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        <Level3 as toql::keyed::Keyed>::key(self).hash(state);
    }
}
impl toql::identity::Identity for Level3 {
    fn columns() -> Vec<String> {
        let mut columns = Vec::with_capacity(2usize);
        columns.push(String::from("id"));
        columns.push(String::from("level2_id"));
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
            "level2_id" => self.level2_id = value.try_into()?,
            _ => {}
        }
        Ok(())
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
                "level4" => {
                    Ok(<Level4 as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)?)
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
                "level4" => {
                    for f in &mut self.level4 {
                        <Level4 as toql::tree::tree_identity::TreeIdentity>::set_id(
                            f,
                            descendents.clone(),
                            action.clone(),
                        )?
                    }
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
                let self_key = <Self as toql::keyed::Keyed>::key(&self);
                let self_key_params = toql::key::Key::params(&self_key);
                let self_key_columns =
                    <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
                for e in &mut self.level4 {
                    let key = <Level4 as toql::keyed::Keyed>::key(e);
                    let merge_key = toql::keyed::Keyed::key(e);
                    let merge_key_params = toql::key::Key::params(&merge_key);
                    let valid = toql::sql_arg::valid_key(&merge_key_params);
                    if matches!(
                        action,
                        toql::tree::tree_identity::IdentityAction::RefreshInvalid
                    ) && valid
                    {
                        continue;
                    }
                    if matches!(
                        action,
                        toql::tree::tree_identity::IdentityAction::RefreshValid
                    ) && !valid
                    {
                        continue;
                    }
                    for (self_key_column, self_key_param) in
                        self_key_columns.iter().zip(&self_key_params)
                    {
                        let calculated_other_column = match self_key_column.as_str() {
                            "id" => "level3_id",
                            x @ _ => x,
                        };
                        if cfg!(debug_assertions) {
                            let foreign_identity_columns =
                                <Level4 as toql::identity::Identity>::columns();
                            if !foreign_identity_columns
                                .contains(&calculated_other_column.to_string())
                            {
                                toql :: tracing :: warn !
                                ("`{}` cannot find column `{}` in `{}`. \
                                                            Try adding `#[toql(foreign_key)]` in `{}` to the missing field.",
                                 "Level3", calculated_other_column, "Level4",
                                 "Level4")
                            }
                        }
                        toql::identity::Identity::set_column(
                            e,
                            calculated_other_column,
                            self_key_param,
                        )?;
                    }
                }
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
        <Level3 as toql::tree::tree_identity::TreeIdentity>::set_id(self, descendents, action)
    }
}
impl toql::tree::tree_map::TreeMap for Level3 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        if registry.get("Level3").is_none() {
            registry.insert_new_mapper::<Level3>()?;
        }
        <Level4 as toql::tree::tree_map::TreeMap>::map(registry)?;
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
                "level4" => {
                    <Level4 as toql::tree::tree_predicate::TreePredicate>::columns(descendents)?
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
                "level4" => {
                    for f in &self.level4 {
                        <Level4 as toql::tree::tree_predicate::TreePredicate>::args(
                            f,
                            descendents.clone(),
                            args,
                        )?
                    }
                }
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
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for Level3
where
    E: std::convert::From<toql::error::ToqlError>,
    Level3Key: toql::from_row::FromRow<R, E>,
    Level4: toql::tree::tree_index::TreeIndex<R, E>,
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
                "level4" => <Level4 as toql::tree::tree_index::TreeIndex<R, E>>::index(
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
                    let fk = Level3Key::from_row(&row, &mut i, &mut iter)?.ok_or(
                        toql::error::ToqlError::ValueMissing(
                            <Level3Key as toql::key::Key>::columns().join(", "),
                        ),
                    )?;
                     println!("Adding row {} for key {:?}", n, &fk);
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
    Level4: toql::tree::tree_index::TreeIndex<R, E>,
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
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for Level3
where
    E: std::convert::From<toql::error::ToqlError>,
    Level3Key: toql::from_row::FromRow<R, E>,
    Level4: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
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
                "level4" => {
                    for f in &mut self.level4 {
                        <Level4 as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                            f,
                            descendents.clone(),
                            &field,
                            rows,
                            row_offset,
                            index,
                            selection_stream,
                        )?
                    }
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
            None => {
                let pk: Level3Key = <Self as toql::keyed::Keyed>::key(&self);
                let mut s = DefaultHasher::new();
                pk.hash(&mut s);
                let h = s.finish();
                let default_vec: Vec<usize> = Vec::new();
                let row_numbers: &Vec<usize> = index.get(&h).unwrap_or(&default_vec);
                println!("Found rows {:?} for primary key {:?}", &row_numbers, &pk);
                let n = row_offset;
                match field {
                    "level4" => {
                        for row_number in row_numbers {
                            let mut i = n;
                            let mut iter = std::iter::repeat(&Select::Query);
                            let row: &R = &rows[*row_number];
                            let fk = Level3Key::from_row(&row, &mut i, &mut iter)?.ok_or(
                                toql::error::ToqlError::ValueMissing("level4".to_string()),
                            )?;
                            if fk == pk {
                                let mut iter = selection_stream.iter();
                                let e = Level4::from_row(&row, &mut i, &mut iter)?.ok_or(
                                    toql::error::ToqlError::ValueMissing("level4".to_string()),
                                )?;
                                self.level4.push(e);
                            }
                        }
                    }
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
    Level4: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
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

impl<R, E> toql::from_row::FromRow<R, E> for Level3Key
where
    E: std::convert::From<toql::error::ToqlError>,
    u64: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(
            0 + <u64 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                + <u64 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?,
        )
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
            level2_id: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Level3Key::level2_id".to_string(),
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
                + <u64 as toql::from_row::FromRow<_, E>>::forward(&mut iter)?
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
            level2_id: {
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
            level4: Vec::new(),
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
    pub fn level2_id(mut self) -> toql::query::field::Field {
        self.0.push_str("level2Id");
        toql::query::field::Field::from(self.0)
    }
    pub fn text(mut self) -> toql::query::field::Field {
        self.0.push_str("text");
        toql::query::field::Field::from(self.0)
    }
    pub fn level4(mut self) -> <Level4 as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("level4_");
        <Level4 as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
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
            "level2Id",
            "level2_id",
            toql::table_mapper::field_options::FieldOptions::new()
                .key(true)
                .preselect(true),
        );
        mapper.map_column_with_options(
            "text",
            "text",
            toql::table_mapper::field_options::FieldOptions::new().preselect(true),
        );
        mapper.map_merge_with_options(
            "level4",
            "Level4",
            {
                {
                    let mut t = toql::sql_expr_macro::sql_expr!("...text = \'ABC");
                    t.extend(toql::sql_expr::SqlExpr::literal(" ")).extend(
                        toql::sql_expr::SqlExpr::from(vec![
                            toql::sql_expr::SqlExprToken::Literal("JOIN ".to_string()),
                            toql::sql_expr::SqlExprToken::Literal("Level3".to_string()),
                            toql::sql_expr::SqlExprToken::Literal(" ".to_string()),
                            toql::sql_expr::SqlExprToken::SelfAlias,
                        ]),
                    );
                    t
                }
            },
            {
                let mut t = toql::sql_expr::SqlExpr::new();
                t.push_self_alias();
                t.push_literal(".");
                t.push_literal("id");
                t.push_literal(" = ");
                t.push_other_alias();
                t.push_literal(".");
                t.push_literal("level3_id");
                t
            },
            toql::table_mapper::merge_options::MergeOptions::new().preselect(true),
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
                "level4" => {
                    return Ok(<Level4 as toql::tree::tree_insert::TreeInsert>::columns(
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
                e.push_literal("level2_id");
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
                "level4" => {
                    for f in &self.level4 {
                        <Level4 as toql::tree::tree_insert::TreeInsert>::values(
                            f,
                            descendents.clone(),
                            roles,
                            should_insert,
                            values,
                        )?
                    }
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
                if !*should_insert.next().unwrap_or(&false) {
                    return Ok(());
                }
                values.push_literal("(");
                values.push_arg(toql::sql_arg::SqlArg::from(&self.id));
                values.push_literal(", ");
                values.push_arg(toql::sql_arg::SqlArg::from(&self.level2_id));
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
                "level4" => {
                    for f in &self.level4 {
                        <Level4 as toql::tree::tree_update::TreeUpdate>::update(
                            f,
                            descendents.clone(),
                            fields,
                            roles,
                            exprs,
                        )?
                    }
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
                if (path_selected || fields.contains("text")) {
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

impl<R, E> toql::toql_api::load::Load<R, E> for Level4
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
impl<R, E> toql::toql_api::load::Load<R, E> for &Level4
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
impl toql::toql_api::count::Count for Level4 where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped
{
}
impl toql::toql_api::count::Count for &Level4 where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped
{
}
impl toql::toql_api::insert::Insert for Level4 where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::toql_api::insert::Insert for &mut Level4 where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::toql_api::update::Update for Level4 where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::toql_api::update::Update for &mut Level4 where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::toql_api::delete::Delete for Level4 where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap
{
}
impl toql::toql_api::delete::Delete for &Level4 where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap
{
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Level4Key {
    pub id: u64,
    pub level3_id: u64,
}
impl toql::key_fields::KeyFields for Level4Key {
    type Entity = Level4;
    fn fields() -> Vec<String> {
        let mut fields: Vec<String> = Vec::new();
        fields.push(String::from("id"));
        fields.push(String::from("level3Id"));
        fields
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params.push(toql::sql_arg::SqlArg::from(&key.level3_id));
        params
    }
}
impl toql::key_fields::KeyFields for &Level4Key {
    type Entity = Level4;
    fn fields() -> Vec<String> {
        <Level4Key as toql::key_fields::KeyFields>::fields()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <Level4Key as toql::key_fields::KeyFields>::params(self)
    }
}
impl toql::key::Key for Level4Key {
    type Entity = Level4;
    fn columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("id"));
        columns.push(String::from("level3_id"));
        columns
    }
    fn default_inverse_columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("level4_id"));
        columns.push(String::from("level4_level3_id"));
        columns
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params.push(toql::sql_arg::SqlArg::from(&key.level3_id));
        params
    }
}
impl toql::key::Key for &Level4Key {
    type Entity = Level4;
    fn columns() -> Vec<String> {
        <Level4Key as toql::key::Key>::columns()
    }
    fn default_inverse_columns() -> Vec<String> {
        <Level4Key as toql::key::Key>::default_inverse_columns()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <Level4Key as toql::key::Key>::params(self)
    }
}
impl toql::keyed::Keyed for Level4 {
    type Key = Level4Key;
    fn key(&self) -> Self::Key {
        Level4Key {
            id: self.id.to_owned(),
            level3_id: self.level3_id.to_owned(),
        }
    }
}
impl toql::keyed::Keyed for &Level4 {
    type Key = Level4Key;
    fn key(&self) -> Self::Key {
        <Level4 as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::Keyed for &mut Level4 {
    type Key = Level4Key;
    fn key(&self) -> Self::Key {
        <Level4 as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::KeyedMut for Level4 {
    fn set_key(&mut self, key: Self::Key) {
        self.id = key.id;
        self.level3_id = key.level3_id;
    }
}
impl toql::keyed::KeyedMut for &mut Level4 {
    fn set_key(&mut self, key: Self::Key) {
        <Level4 as toql::keyed::KeyedMut>::set_key(self, key)
    }
}
impl std::convert::TryFrom<Vec<toql::sql_arg::SqlArg>> for Level4Key {
    type Error = toql::error::ToqlError;
    fn try_from(args: Vec<toql::sql_arg::SqlArg>) -> toql::result::Result<Self> {
        use std::convert::TryInto;
        Ok(Level4Key {
            id: args
                .get(0)
                .ok_or(toql::error::ToqlError::ValueMissing("id".to_string()))?
                .try_into()?,
            level3_id: args
                .get(1)
                .ok_or(toql::error::ToqlError::ValueMissing(
                    "level3_id".to_string(),
                ))?
                .try_into()?,
        })
    }
}
impl std::convert::From<(u64, u64)> for Level4Key {
    fn from(key: (u64, u64)) -> Self {
        Self {
            id: key.0,
            level3_id: key.1,
        }
    }
}
impl std::hash::Hash for Level4 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        <Level4 as toql::keyed::Keyed>::key(self).hash(state);
    }
}
impl toql::identity::Identity for Level4 {
    fn columns() -> Vec<String> {
        let mut columns = Vec::with_capacity(2usize);
        columns.push(String::from("id"));
        columns.push(String::from("level3_id"));
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
            "level3_id" => self.level3_id = value.try_into()?,
            _ => {}
        }
        Ok(())
    }
}

impl toql::tree::tree_identity::TreeIdentity for Level4 {
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
                    entity: &mut Level4,
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
                        <<Level4 as toql::keyed::Keyed>::Key as toql::key::Key>::columns().len();
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
                let self_key = <Self as toql::keyed::Keyed>::key(&self);
                let self_key_params = toql::key::Key::params(&self_key);
                let self_key_columns =
                    <<Self as toql::keyed::Keyed>::Key as toql::key::Key>::columns();
            }
        }
        Ok(())
    }
}
impl toql::tree::tree_identity::TreeIdentity for &mut Level4 {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level4 as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)
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
        <Level4 as toql::tree::tree_identity::TreeIdentity>::set_id(self, descendents, action)
    }
}
impl toql::tree::tree_map::TreeMap for Level4 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        if registry.get("Level4").is_none() {
            registry.insert_new_mapper::<Level4>()?;
        }
        Ok(())
    }
}
impl toql::tree::tree_map::TreeMap for &Level4 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        <Level4 as toql::tree::tree_map::TreeMap>::map(registry)
    }
}
impl toql::tree::tree_map::TreeMap for &mut Level4 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        <Level4 as toql::tree::tree_map::TreeMap>::map(registry)
    }
}
impl toql::tree::tree_predicate::TreePredicate for Level4 {
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
impl toql::tree::tree_predicate::TreePredicate for &Level4 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level4 as toql::tree::tree_predicate::TreePredicate>::columns(descendents)
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
        <Level4 as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl toql::tree::tree_predicate::TreePredicate for &mut Level4 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level4 as toql::tree::tree_predicate::TreePredicate>::columns(descendents)
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
        <Level4 as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for Level4
where
    E: std::convert::From<toql::error::ToqlError>,
    Level4Key: toql::from_row::FromRow<R, E>,
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
                    let fk = Level4Key::from_row(&row, &mut i, &mut iter)?.ok_or(
                        toql::error::ToqlError::ValueMissing(
                            <Level4Key as toql::key::Key>::columns().join(", "),
                        ),
                    )?;
                     println!("Adding row {} for key {:?}", n, &fk);
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
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for &Level4
where
    E: std::convert::From<toql::error::ToqlError>,
    Level4Key: toql::from_row::FromRow<R, E>,
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
        <Level4 as toql::tree::tree_index::TreeIndex<R, E>>::index(
            descendents,
            rows,
            row_offset,
            index,
        )
    }
}
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for Level4
where
    E: std::convert::From<toql::error::ToqlError>,
    Level4Key: toql::from_row::FromRow<R, E>,
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
                let pk: Level4Key = <Self as toql::keyed::Keyed>::key(&self);
                let mut s = DefaultHasher::new();
                pk.hash(&mut s);
                let h = s.finish();
                let default_vec: Vec<usize> = Vec::new();
                let row_numbers: &Vec<usize> = index.get(&h).unwrap_or(&default_vec);
                 println!("Found rows {:?} for primary key {:?}", &row_numbers, &pk);
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
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for &mut Level4
where
    E: std::convert::From<toql::error::ToqlError>,
    Level4Key: toql::from_row::FromRow<R, E>,
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
        <Level4 as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
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

impl<R, E> toql::from_row::FromRow<R, E> for Level4Key
where
    E: std::convert::From<toql::error::ToqlError>,
    u64: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(
            0 + <u64 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                + <u64 as toql::from_row::FromRow<R, E>>::forward(&mut iter)?,
        )
    }
    #[allow(unused_variables, unused_mut)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Level4Key>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        Ok(Some(Level4Key {
            id: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Level4Key::id".to_string(),
                    )
                    .into(),
                )?
            },
            level3_id: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Level4Key::level3_id".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Level4
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
                + <u64 as toql::from_row::FromRow<_, E>>::forward(&mut iter)?
                + <String as toql::from_row::FromRow<_, E>>::forward(&mut iter)?,
        )
    }
    #[allow(unused_variables, unused_mut, unused_imports)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Level4>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        use toql::sql_builder::select_stream::Select;
        Ok(Some(Level4 {
            id: {
                match <u64 as toql::from_row::FromRow<_, E>>::from_row(row, i, iter)? {
                    Some(s) => s,
                    _ => return Ok(None),
                }
            },
            level3_id: {
                match <u64 as toql::from_row::FromRow<_, E>>::from_row(row, i, iter)? {
                    Some(s) => s,
                    _ => return Ok(None),
                }
            },
            text: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Level4::text".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl toql::query_fields::QueryFields for Level4 {
    type FieldsType = Level4Fields;
    fn fields() -> Level4Fields {
        Level4Fields::new()
    }
    fn fields_from_path(path: String) -> Level4Fields {
        Level4Fields::from_path(path)
    }
}
pub struct Level4Fields(String);
impl toql::query_path::QueryPath for Level4Fields {
    fn into_path(self) -> String {
        self.0
    }
}
impl Level4Fields {
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
    pub fn level3_id(mut self) -> toql::query::field::Field {
        self.0.push_str("level3Id");
        toql::query::field::Field::from(self.0)
    }
    pub fn text(mut self) -> toql::query::field::Field {
        self.0.push_str("text");
        toql::query::field::Field::from(self.0)
    }
}

impl toql::table_mapper::mapped::Mapped for Level4 {
    fn type_name() -> String {
        String::from("Level4")
    }
    fn table_name() -> String {
        String::from("Level4")
    }
    fn table_alias() -> String {
        String::from("level4")
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
            "level3Id",
            "level3_id",
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
impl toql::table_mapper::mapped::Mapped for &Level4 {
    fn type_name() -> String {
        <Level4 as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Level4 as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Level4 as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Level4 as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}
impl toql::table_mapper::mapped::Mapped for &mut Level4 {
    fn type_name() -> String {
        <Level4 as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Level4 as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Level4 as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Level4 as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}

impl toql::tree::tree_insert::TreeInsert for Level4 {
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
                e.push_literal("level3_id");
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
                values.push_arg(toql::sql_arg::SqlArg::from(&self.level3_id));
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
impl toql::tree::tree_insert::TreeInsert for &Level4 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level4 as toql::tree::tree_insert::TreeInsert>::columns(descendents)
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
        <Level4 as toql::tree::tree_insert::TreeInsert>::values(
            self,
            descendents,
            roles,
            should_insert,
            values,
        )
    }
}
impl toql::tree::tree_insert::TreeInsert for &mut Level4 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Level4 as toql::tree::tree_insert::TreeInsert>::columns(descendents)
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
        <Level4 as toql::tree::tree_insert::TreeInsert>::values(
            self,
            descendents,
            roles,
            should_insert,
            values,
        )
    }
}

impl toql::tree::tree_update::TreeUpdate for Level4 {
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
                expr.push_literal("Level4");
                expr.push_literal(" SET ");
                let tokens = expr.tokens().len();
                if (path_selected || fields.contains("text")) {
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
                        toql::sql_expr::resolver::Resolver::new().with_self_alias("Level4");
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
impl toql::tree::tree_update::TreeUpdate for &mut Level4 {
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
        <Level4 as toql::tree::tree_update::TreeUpdate>::update(
            self,
            descendents,
            fields,
            roles,
            exprs,
        )
    }
}
 */