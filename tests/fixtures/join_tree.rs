use toql::prelude::{Join, Toql};

#[derive(Debug, PartialEq, Toql)]
pub struct Alpha {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(join())]
    beta1: Beta,  // Preselected inner join

    #[toql(join())]
    beta2: Join<Beta>, // Preselected inner join with Join wrapper

    #[toql(join())]
    beta3: Option<Beta>,  // Selectable inner join

    #[toql(join())]
    beta4: Option<Join<Beta>>, // Selectable inner join with Join wrapper

    #[toql(join())]
    beta5: Option<Option<Beta>>,  // Selectable left join

    #[toql(join())]
    beta6: Option<Option<Join<Beta>>>, // Selectable left join with Join wrapper

     #[toql(preselect, join())]
    beta7: Option<Beta>,  // Preselected left join

    #[toql(preselect, join())]
    beta8: Option<Join<Beta>>, // Preselected left join with Join wrapper
}

#[derive(Debug, PartialEq, Toql)]
pub struct Beta {
    #[toql(key)]
    id: u64,
    text: String,
}

#[derive(Debug, PartialEq, Toql)]
pub struct Alpha1 {
    #[toql(key, join())] // With joined key (always inner join and preselected)
    id: Beta,
    text: String,
}

#[derive(Debug, PartialEq, Toql)]
pub struct Alpha2 {
    #[toql(key, join())] // With joined key (always inner join and preselected)
    id: Join<Beta>,
    text: String,
}
/*
#[derive(Debug, PartialEq)]
pub struct Alpha {
    id: u64,
    text: String,

    beta1: Beta, // Preselected inner join

    beta2: Join<Beta>, // Preselected inner join with Join wrapper

    beta3: Option<Beta>, // Selectable inner join

    beta4: Option<Join<Beta>>, // Selectable inner join with Join wrapper

    beta5: Option<Option<Beta>>, // Selectable left join

    beta6: Option<Option<Join<Beta>>>, // Selectable left join with Join wrapper

    beta7: Option<Beta>, // Preselected left join

    beta8: Option<Join<Beta>>, // Preselected left join with Join wrapper
}

#[derive(Debug, PartialEq)]
pub struct Alpha1 {
    id: Beta,
    text: String,
}

#[derive(Debug, PartialEq)]
pub struct Alpha2 {
    id: Join<Beta>,
    text: String,
}

#[derive(Debug, PartialEq)]
pub struct Beta {
    id: u64,
    text: String,
}*/
/* 
impl<R, E> toql::backend::Load<R, E> for Alpha1
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>
        + std::fmt::Debug,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl<R, E> toql::backend::Load<R, E> for &Alpha1
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>
        + std::fmt::Debug,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl toql::backend::Insert for Alpha1 where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::Insert for &mut Alpha1 where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::Update for Alpha1 where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::Update for &mut Alpha1 where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::Count for Alpha1 where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::Count for &Alpha1 where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::Delete for Alpha1 where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
{
}
impl toql::backend::Delete for &Alpha1 where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
{
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Alpha1Key {
    pub id: <Beta as toql::keyed::Keyed>::Key,
}
impl toql::key_fields::KeyFields for Alpha1Key {
    type Entity = Alpha1;
    fn fields() -> Vec<String> {
        let mut fields: Vec<String> = Vec::new();
        <<Beta as toql::keyed::Keyed>::Key as toql::key_fields::KeyFields>::fields()
            .iter()
            .for_each(|other_field| {
                fields.push(format!("{}_{}", "id", other_field));
            });
        fields
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.extend_from_slice(&toql::key::Key::params(&key.id));
        params
    }
}
impl toql::key_fields::KeyFields for &Alpha1Key {
    type Entity = Alpha1;
    fn fields() -> Vec<String> {
        <Alpha1Key as toql::key_fields::KeyFields>::fields()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <Alpha1Key as toql::key_fields::KeyFields>::params(self)
    }
}
impl toql::key::Key for Alpha1Key {
    type Entity = Alpha1;
    fn columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
            .iter()
            .for_each(|other_column| {
                let default_self_column = format!("id_{}", other_column);
                let column = {
                    let self_column = match other_column.as_str() {
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
        <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::default_inverse_columns()
            .iter()
            .for_each(|other_column| {
                let default_self_column = format!("id_{}", other_column);
                let column = {
                    let self_column = match other_column.as_str() {
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
        params.extend_from_slice(&toql::key::Key::params(&key.id));
        params
    }
}
impl toql::key::Key for &Alpha1Key {
    type Entity = Alpha1;
    fn columns() -> Vec<String> {
        <Alpha1Key as toql::key::Key>::columns()
    }
    fn default_inverse_columns() -> Vec<String> {
        <Alpha1Key as toql::key::Key>::default_inverse_columns()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <Alpha1Key as toql::key::Key>::params(self)
    }
}
impl toql::keyed::Keyed for Alpha1 {
    type Key = Alpha1Key;
    fn key(&self) -> Self::Key {
        Alpha1Key {
            id: <Beta as toql::keyed::Keyed>::key(&self.id),
        }
    }
}
impl toql::keyed::Keyed for &Alpha1 {
    type Key = Alpha1Key;
    fn key(&self) -> Self::Key {
        <Alpha1 as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::Keyed for &mut Alpha1 {
    type Key = Alpha1Key;
    fn key(&self) -> Self::Key {
        <Alpha1 as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::KeyedMut for Alpha1 {
    fn set_key(&mut self, key: Self::Key) {
        <Beta as toql::keyed::KeyedMut>::set_key(&mut self.id, key.id);
    }
}
impl toql::keyed::KeyedMut for &mut Alpha1 {
    fn set_key(&mut self, key: Self::Key) {
        <Alpha1 as toql::keyed::KeyedMut>::set_key(self, key)
    }
}
impl std::convert::TryFrom<Vec<toql::sql_arg::SqlArg>> for Alpha1Key {
    type Error = toql::error::ToqlError;
    fn try_from(args: Vec<toql::sql_arg::SqlArg>) -> toql::result::Result<Self> {
        use std::convert::TryInto;
        Ok(Alpha1Key {
            id: <Beta as toql::keyed::Keyed>::Key::try_from(Vec::from(&args[0..]))?,
        })
    }
}
impl std::convert::From<<Beta as toql::keyed::Keyed>::Key> for Alpha1Key {
    fn from(key: <Beta as toql::keyed::Keyed>::Key) -> Self {
        Self { id: key }
    }
}
impl std::hash::Hash for Alpha1 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        <Alpha1 as toql::keyed::Keyed>::key(self).hash(state);
    }
}

impl toql::tree::tree_identity::TreeIdentity for Alpha1 {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: &mut I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => Ok(<Beta as toql::tree::tree_identity::TreeIdentity>::auto_id(
                    &mut descendents,
                )?),
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
        mut descendents: &mut I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => <Beta as toql::tree::tree_identity::TreeIdentity>::set_id(
                    &mut self.id,
                    &mut descendents,
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
            None => {}
        }
        Ok(())
    }
}
impl toql::tree::tree_identity::TreeIdentity for &mut Alpha1 {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: &mut I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)
    }
    #[allow(unused_mut)]
    fn set_id<'a, 'b, I>(
        &mut self,
        mut descendents: &mut I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_identity::TreeIdentity>::set_id(self, descendents, action)
    }
}
impl toql::tree::tree_map::TreeMap for Alpha1 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        if registry.get("Alpha1").is_none() {
            registry.insert_new_mapper::<Alpha1>()?;
        }
        <Beta as toql::tree::tree_map::TreeMap>::map(registry)?;
        Ok(())
    }
}
impl toql::tree::tree_map::TreeMap for &Alpha1 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        <Alpha1 as toql::tree::tree_map::TreeMap>::map(registry)
    }
}
impl toql::tree::tree_predicate::TreePredicate for Alpha1 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        Ok(match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => <Beta as toql::tree::tree_predicate::TreePredicate>::columns(
                    &self.id,
                    &mut descendents,
                )?,
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
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => <Beta as toql::tree::tree_predicate::TreePredicate>::args(
                    &self.id,
                    &mut descendents,
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
impl toql::tree::tree_predicate::TreePredicate for &Alpha1 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_predicate::TreePredicate>::columns(self, descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl toql::tree::tree_predicate::TreePredicate for &mut Alpha1 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_predicate::TreePredicate>::columns(self, descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for Alpha1
where
    E: std::convert::From<toql::error::ToqlError>,
    Alpha1Key: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_index::TreeIndex<R, E>,
{
    #[allow(unused_variables, unused_mut)]
    fn index<'a, I>(
        mut descendents: &mut I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        use toql::from_row::FromRow;
        use toql::sql_builder::select_stream::Select;
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => <Beta as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    &mut descendents,
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
                    let fk = Alpha1Key::from_row(&row, &mut i, &mut iter)?.ok_or(
                        toql::error::ToqlError::ValueMissing(
                            <Alpha1Key as toql::key::Key>::columns().join(", "),
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
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for &Alpha1
where
    E: std::convert::From<toql::error::ToqlError>,
    Alpha1Key: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_index::TreeIndex<R, E>,
{
    #[allow(unused_mut)]
    fn index<'a, I>(
        mut descendents: &mut I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_index::TreeIndex<R, E>>::index(
            descendents,
            rows,
            row_offset,
            index,
        )
    }
}
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for Alpha1
where
    E: std::convert::From<toql::error::ToqlError>,
    Alpha1Key: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
{
    #[allow(unreachable_code, unused_variables, unused_mut)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: &mut I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        use toql::from_row::FromRow;
        use toql::keyed::Keyed;
        use toql::sql_builder::select_stream::Select;
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => <Beta as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                    &mut self.id,
                    &mut descendents,
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
                let pk: Alpha1Key = <Self as toql::keyed::Keyed>::key(&self);
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
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for &mut Alpha1
where
    E: std::convert::From<toql::error::ToqlError>,
    Alpha1Key: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
{
    #[allow(unused_mut)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: &mut I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
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

impl<R, E> toql::from_row::FromRow<R, E> for Alpha1Key
where
    E: std::convert::From<toql::error::ToqlError>,
    Beta: toql::from_row::FromRow<R, E> + toql::keyed::Keyed,
    <Beta as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(0 + <Beta as toql::from_row::FromRow<R, E>>::forward(&mut iter)?)
    }
    #[allow(unused_variables, unused_mut)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Alpha1Key>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        Ok(Some(Alpha1Key {
            id: {
                <<Beta as toql::keyed::Keyed>::Key>::from_row(row, i, iter)?.ok_or(
                    toql::error::ToqlError::ValueMissing("Alpha1Key::id".to_string()),
                )?
            },
        }))
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Alpha1
where
    E: std::convert::From<toql::error::ToqlError>,
    Beta: toql::from_row::FromRow<R, E>,
    String: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(0 + if iter
            .next()
            .ok_or(toql::error::ToqlError::DeserializeError(
                toql::deserialize::error::DeserializeError::StreamEnd,
            ))?
            .is_selected()
        {
            <Beta as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
        } else {
            0
        } + <String as toql::from_row::FromRow<_, E>>::forward(&mut iter)?)
    }
    #[allow(unused_variables, unused_mut)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Alpha1>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        use toql::sql_builder::select_stream::Select;
        Ok(Some(Alpha1 {
            id: {
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
                                toql::error::ToqlError::ValueMissing("id".to_string()).into()
                            )
                        }
                    }
                } else {
                    return Err(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::SelectionExpected(
                            "id".to_string(),
                        ),
                    )
                    .into());
                }
            },
            text: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Alpha1::text".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl toql::query_fields::QueryFields for Alpha1 {
    type FieldsType = Alpha1Fields;
    fn fields() -> Alpha1Fields {
        Alpha1Fields::new()
    }
    fn fields_from_path(path: String) -> Alpha1Fields {
        Alpha1Fields::from_path(path)
    }
}
pub struct Alpha1Fields(String);
impl toql::query_path::QueryPath for Alpha1Fields {
    fn into_path(self) -> String {
        self.0
    }
}
impl Alpha1Fields {
    pub fn new() -> Self {
        Self::from_path(String::from(""))
    }
    pub fn from_path(path: String) -> Self {
        Self(path)
    }
    pub fn into_name(self) -> String {
        self.0
    }
    pub fn id(mut self) -> <Beta as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("id_");
        <Beta as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
    pub fn text(mut self) -> toql::query::field::Field {
        self.0.push_str("text");
        toql::query::field::Field::from(self.0)
    }
}

impl toql::table_mapper::mapped::Mapped for Alpha1 {
    fn type_name() -> String {
        String::from("Alpha1")
    }
    fn table_name() -> String {
        String::from("Alpha1")
    }
    fn table_alias() -> String {
        String::from("alpha1")
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        mapper.map_join_with_options(
            "id",
            "Beta",
            toql::table_mapper::join_type::JoinType::Inner,
            {
                let mut t = toql::sql_expr::SqlExpr::literal(
                    <Beta as toql::table_mapper::mapped::Mapped>::table_name(),
                );
                t.push_literal(" ");
                t.push_other_alias();
                t
            },
            {
                let mut t = toql::sql_expr::SqlExpr::new();
                <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                    .iter()
                    .for_each(|other_column| {
                        let default_self_column = format!("id_{}", other_column);
                        let self_column = {
                            let self_column = match other_column.as_str() {
                                _ => &default_self_column,
                            };
                            self_column
                        };
                        t.push_self_alias();
                        t.push_literal(".");
                        t.push_literal(self_column);
                        t.push_literal(" = ");
                        t.push_other_alias();
                        t.push_literal(".");
                        t.push_literal(other_column);
                        t.push_literal(" AND ");
                    });
                t.pop_literals(5);
                t
            },
            toql::table_mapper::join_options::JoinOptions::new()
                .preselect(true)
                .key(true),
        );
        mapper.map_column_with_options(
            "text",
            "text",
            toql::table_mapper::field_options::FieldOptions::new().preselect(true),
        );
        Ok(())
    }
}
impl toql::table_mapper::mapped::Mapped for &Alpha1 {
    fn type_name() -> String {
        <Alpha1 as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Alpha1 as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Alpha1 as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Alpha1 as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}
impl toql::table_mapper::mapped::Mapped for &mut Alpha1 {
    fn type_name() -> String {
        <Alpha1 as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Alpha1 as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Alpha1 as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Alpha1 as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}

impl toql::tree::tree_insert::TreeInsert for Alpha1 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        let mut e = toql::sql_expr::SqlExpr::new();
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => {
                    return Ok(<Beta as toql::tree::tree_insert::TreeInsert>::columns(
                        &mut descendents,
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
                for other_column in <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("id_{}", other_column);
                    let self_column = {
                        let self_column = match other_column.as_str() {
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
    fn values<'a, I>(
        &self,
        mut descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => <Beta as toql::tree::tree_insert::TreeInsert>::values(
                    &self.id,
                    &mut descendents,
                    roles,
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
                values.push_literal("(");
                &toql::key::Key::params(&<Beta as toql::keyed::Keyed>::key(&self.id))
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
impl toql::tree::tree_insert::TreeInsert for &Alpha1 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, I>(
        &self,
        mut descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, values)
    }
}
impl toql::tree::tree_insert::TreeInsert for &mut Alpha1 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, I>(
        &self,
        mut descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, values)
    }
}

impl toql::tree::tree_update::TreeUpdate for Alpha1 {
    #[allow(unused_mut, unused_variables, unused_parens)]
    fn update<'a, I>(
        &self,
        mut descendents: &mut I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
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
                let path_selected = fields.contains("*");
                let mut expr = toql::sql_expr::SqlExpr::new();
                expr.push_literal("UPDATE ");
                expr.push_literal("Alpha1");
                expr.push_literal(" ");
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
                        toql::sql_expr::resolver::Resolver::new().with_self_alias("Alpha1");
                    expr.extend(resolver.alias_to_literals(&toql::key::Key::predicate_expr(&key))?);
                    exprs.push(expr);
                }
            }
        };
        Ok(())
    }
}
impl toql::tree::tree_update::TreeUpdate for &mut Alpha1 {
    #[allow(unused_mut)]
    fn update<'a, I>(
        &self,
        mut descendents: &mut I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha1 as toql::tree::tree_update::TreeUpdate>::update(
            self,
            descendents,
            fields,
            roles,
            exprs,
        )
    }
}

impl<R, E> toql::backend::Load<R, E> for Beta
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>
        + std::fmt::Debug,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl<R, E> toql::backend::Load<R, E> for &Beta
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>
        + std::fmt::Debug,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl toql::backend::Insert for Beta where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::Insert for &mut Beta where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::Update for Beta where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::Update for &mut Beta where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::Count for Beta where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::Count for &Beta where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::Delete for Beta where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
{
}
impl toql::backend::Delete for &Beta where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
{
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct BetaKey {
    pub id: u64,
}
impl toql::key_fields::KeyFields for BetaKey {
    type Entity = Beta;
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
impl toql::key_fields::KeyFields for &BetaKey {
    type Entity = Beta;
    fn fields() -> Vec<String> {
        <BetaKey as toql::key_fields::KeyFields>::fields()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <BetaKey as toql::key_fields::KeyFields>::params(self)
    }
}
impl toql::key::Key for BetaKey {
    type Entity = Beta;
    fn columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("id"));
        columns
    }
    fn default_inverse_columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("beta_id"));
        columns
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params
    }
}
impl toql::key::Key for &BetaKey {
    type Entity = Beta;
    fn columns() -> Vec<String> {
        <BetaKey as toql::key::Key>::columns()
    }
    fn default_inverse_columns() -> Vec<String> {
        <BetaKey as toql::key::Key>::default_inverse_columns()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <BetaKey as toql::key::Key>::params(self)
    }
}
impl From<BetaKey> for toql::sql_arg::SqlArg {
    fn from(t: BetaKey) -> toql::sql_arg::SqlArg {
        toql::sql_arg::SqlArg::from(t.id)
    }
}
impl From<&BetaKey> for toql::sql_arg::SqlArg {
    fn from(t: &BetaKey) -> toql::sql_arg::SqlArg {
        toql::sql_arg::SqlArg::from(t.id.to_owned())
    }
}
impl toql::keyed::Keyed for Beta {
    type Key = BetaKey;
    fn key(&self) -> Self::Key {
        BetaKey {
            id: self.id.to_owned(),
        }
    }
}
impl toql::keyed::Keyed for &Beta {
    type Key = BetaKey;
    fn key(&self) -> Self::Key {
        <Beta as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::Keyed for &mut Beta {
    type Key = BetaKey;
    fn key(&self) -> Self::Key {
        <Beta as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::KeyedMut for Beta {
    fn set_key(&mut self, key: Self::Key) {
        self.id = key.id;
    }
}
impl toql::keyed::KeyedMut for &mut Beta {
    fn set_key(&mut self, key: Self::Key) {
        <Beta as toql::keyed::KeyedMut>::set_key(self, key)
    }
}
impl std::convert::TryFrom<Vec<toql::sql_arg::SqlArg>> for BetaKey {
    type Error = toql::error::ToqlError;
    fn try_from(args: Vec<toql::sql_arg::SqlArg>) -> toql::result::Result<Self> {
        use std::convert::TryInto;
        Ok(BetaKey {
            id: args
                .get(0)
                .ok_or(toql::error::ToqlError::ValueMissing("id".to_string()))?
                .try_into()?,
        })
    }
}
impl std::convert::From<u64> for BetaKey {
    fn from(key: u64) -> Self {
        Self { id: key }
    }
}
impl std::hash::Hash for Beta {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        <Beta as toql::keyed::Keyed>::key(self).hash(state);
    }
}

impl toql::tree::tree_identity::TreeIdentity for Beta {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: &mut I) -> std::result::Result<bool, toql::error::ToqlError>
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
        mut descendents: &mut I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
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
            None => {}
        }
        Ok(())
    }
}
impl toql::tree::tree_identity::TreeIdentity for &mut Beta {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: &mut I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)
    }
    #[allow(unused_mut)]
    fn set_id<'a, 'b, I>(
        &mut self,
        mut descendents: &mut I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_identity::TreeIdentity>::set_id(self, descendents, action)
    }
}
impl toql::tree::tree_map::TreeMap for Beta {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        if registry.get("Beta").is_none() {
            registry.insert_new_mapper::<Beta>()?;
        }
        Ok(())
    }
}
impl toql::tree::tree_map::TreeMap for &Beta {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        <Beta as toql::tree::tree_map::TreeMap>::map(registry)
    }
}
impl toql::tree::tree_predicate::TreePredicate for Beta {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
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
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
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
impl toql::tree::tree_predicate::TreePredicate for &Beta {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_predicate::TreePredicate>::columns(self, descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl toql::tree::tree_predicate::TreePredicate for &mut Beta {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_predicate::TreePredicate>::columns(self, descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for Beta
where
    E: std::convert::From<toql::error::ToqlError>,
    BetaKey: toql::from_row::FromRow<R, E>,
{
    #[allow(unused_variables, unused_mut)]
    fn index<'a, I>(
        mut descendents: &mut I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
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
                    let fk = BetaKey::from_row(&row, &mut i, &mut iter)?.ok_or(
                        toql::error::ToqlError::ValueMissing(
                            <BetaKey as toql::key::Key>::columns().join(", "),
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
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for &Beta
where
    E: std::convert::From<toql::error::ToqlError>,
    BetaKey: toql::from_row::FromRow<R, E>,
{
    #[allow(unused_mut)]
    fn index<'a, I>(
        mut descendents: &mut I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_index::TreeIndex<R, E>>::index(
            descendents,
            rows,
            row_offset,
            index,
        )
    }
}
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for Beta
where
    E: std::convert::From<toql::error::ToqlError>,
    BetaKey: toql::from_row::FromRow<R, E>,
{
    #[allow(unreachable_code, unused_variables, unused_mut)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: &mut I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
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
                let pk: BetaKey = <Self as toql::keyed::Keyed>::key(&self);
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
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for &mut Beta
where
    E: std::convert::From<toql::error::ToqlError>,
    BetaKey: toql::from_row::FromRow<R, E>,
{
    #[allow(unused_mut)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: &mut I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
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

impl<R, E> toql::from_row::FromRow<R, E> for BetaKey
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
    ) -> std::result::Result<Option<BetaKey>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        Ok(Some(BetaKey {
            id: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "BetaKey::id".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Beta
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
                + <String as toql::from_row::FromRow<_, E>>::forward(&mut iter)?,
        )
    }
    #[allow(unused_variables, unused_mut)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Beta>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        use toql::sql_builder::select_stream::Select;
        Ok(Some(Beta {
            id: {
                match <u64 as toql::from_row::FromRow<_, E>>::from_row(row, i, iter)? {
                    Some(s) => s,
                    _ => return Ok(None),
                }
            },
            text: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Beta::text".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl toql::query_fields::QueryFields for Beta {
    type FieldsType = BetaFields;
    fn fields() -> BetaFields {
        BetaFields::new()
    }
    fn fields_from_path(path: String) -> BetaFields {
        BetaFields::from_path(path)
    }
}
pub struct BetaFields(String);
impl toql::query_path::QueryPath for BetaFields {
    fn into_path(self) -> String {
        self.0
    }
}
impl BetaFields {
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

impl toql::table_mapper::mapped::Mapped for Beta {
    fn type_name() -> String {
        String::from("Beta")
    }
    fn table_name() -> String {
        String::from("Beta")
    }
    fn table_alias() -> String {
        String::from("beta")
    }
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
impl toql::table_mapper::mapped::Mapped for &Beta {
    fn type_name() -> String {
        <Beta as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Beta as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Beta as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Beta as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}
impl toql::table_mapper::mapped::Mapped for &mut Beta {
    fn type_name() -> String {
        <Beta as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Beta as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Beta as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Beta as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}

impl toql::tree::tree_insert::TreeInsert for Beta {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
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
    fn values<'a, I>(
        &self,
        mut descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
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
impl toql::tree::tree_insert::TreeInsert for &Beta {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, I>(
        &self,
        mut descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, values)
    }
}
impl toql::tree::tree_insert::TreeInsert for &mut Beta {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, I>(
        &self,
        mut descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, values)
    }
}

impl toql::tree::tree_update::TreeUpdate for Beta {
    #[allow(unused_mut, unused_variables, unused_parens)]
    fn update<'a, I>(
        &self,
        mut descendents: &mut I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
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
                let path_selected = fields.contains("*");
                let mut expr = toql::sql_expr::SqlExpr::new();
                expr.push_literal("UPDATE ");
                expr.push_literal("Beta");
                expr.push_literal(" ");
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
                        toql::sql_expr::resolver::Resolver::new().with_self_alias("Beta");
                    expr.extend(resolver.alias_to_literals(&toql::key::Key::predicate_expr(&key))?);
                    exprs.push(expr);
                }
            }
        };
        Ok(())
    }
}
impl toql::tree::tree_update::TreeUpdate for &mut Beta {
    #[allow(unused_mut)]
    fn update<'a, I>(
        &self,
        mut descendents: &mut I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Beta as toql::tree::tree_update::TreeUpdate>::update(
            self,
            descendents,
            fields,
            roles,
            exprs,
        )
    }
}

impl<R, E> toql::backend::Load<R, E> for Alpha
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>
        + std::fmt::Debug,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl<R, E> toql::backend::Load<R, E> for &Alpha
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>
        + std::fmt::Debug,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl toql::backend::Insert for Alpha where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::Insert for &mut Alpha where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::Update for Alpha where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::Update for &mut Alpha where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::Count for Alpha where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::Count for &Alpha where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::Delete for Alpha where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
{
}
impl toql::backend::Delete for &Alpha where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
{
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct AlphaKey {
    pub id: u64,
}
impl toql::key_fields::KeyFields for AlphaKey {
    type Entity = Alpha;
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
impl toql::key_fields::KeyFields for &AlphaKey {
    type Entity = Alpha;
    fn fields() -> Vec<String> {
        <AlphaKey as toql::key_fields::KeyFields>::fields()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <AlphaKey as toql::key_fields::KeyFields>::params(self)
    }
}
impl toql::key::Key for AlphaKey {
    type Entity = Alpha;
    fn columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("id"));
        columns
    }
    fn default_inverse_columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("alpha_id"));
        columns
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params
    }
}
impl toql::key::Key for &AlphaKey {
    type Entity = Alpha;
    fn columns() -> Vec<String> {
        <AlphaKey as toql::key::Key>::columns()
    }
    fn default_inverse_columns() -> Vec<String> {
        <AlphaKey as toql::key::Key>::default_inverse_columns()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <AlphaKey as toql::key::Key>::params(self)
    }
}
impl From<AlphaKey> for toql::sql_arg::SqlArg {
    fn from(t: AlphaKey) -> toql::sql_arg::SqlArg {
        toql::sql_arg::SqlArg::from(t.id)
    }
}
impl From<&AlphaKey> for toql::sql_arg::SqlArg {
    fn from(t: &AlphaKey) -> toql::sql_arg::SqlArg {
        toql::sql_arg::SqlArg::from(t.id.to_owned())
    }
}
impl toql::keyed::Keyed for Alpha {
    type Key = AlphaKey;
    fn key(&self) -> Self::Key {
        AlphaKey {
            id: self.id.to_owned(),
        }
    }
}
impl toql::keyed::Keyed for &Alpha {
    type Key = AlphaKey;
    fn key(&self) -> Self::Key {
        <Alpha as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::Keyed for &mut Alpha {
    type Key = AlphaKey;
    fn key(&self) -> Self::Key {
        <Alpha as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::KeyedMut for Alpha {
    fn set_key(&mut self, key: Self::Key) {
        self.id = key.id;
    }
}
impl toql::keyed::KeyedMut for &mut Alpha {
    fn set_key(&mut self, key: Self::Key) {
        <Alpha as toql::keyed::KeyedMut>::set_key(self, key)
    }
}
impl std::convert::TryFrom<Vec<toql::sql_arg::SqlArg>> for AlphaKey {
    type Error = toql::error::ToqlError;
    fn try_from(args: Vec<toql::sql_arg::SqlArg>) -> toql::result::Result<Self> {
        use std::convert::TryInto;
        Ok(AlphaKey {
            id: args
                .get(0)
                .ok_or(toql::error::ToqlError::ValueMissing("id".to_string()))?
                .try_into()?,
        })
    }
}
impl std::convert::From<u64> for AlphaKey {
    fn from(key: u64) -> Self {
        Self { id: key }
    }
}
impl std::hash::Hash for Alpha {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        <Alpha as toql::keyed::Keyed>::key(self).hash(state);
    }
}

impl toql::tree::tree_identity::TreeIdentity for Alpha {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: &mut I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "beta1" => Ok(<Beta as toql::tree::tree_identity::TreeIdentity>::auto_id(
                    &mut descendents,
                )?),
                "beta2" => Ok(
                    <Join<Beta> as toql::tree::tree_identity::TreeIdentity>::auto_id(
                        &mut descendents,
                    )?,
                ),
                "beta3" => Ok(<Beta as toql::tree::tree_identity::TreeIdentity>::auto_id(
                    &mut descendents,
                )?),
                "beta4" => Ok(
                    <Join<Beta> as toql::tree::tree_identity::TreeIdentity>::auto_id(
                        &mut descendents,
                    )?,
                ),
                "beta5" => Ok(<Beta as toql::tree::tree_identity::TreeIdentity>::auto_id(
                    &mut descendents,
                )?),
                "beta6" => Ok(
                    <Join<Beta> as toql::tree::tree_identity::TreeIdentity>::auto_id(
                        &mut descendents,
                    )?,
                ),
                "beta7" => Ok(<Beta as toql::tree::tree_identity::TreeIdentity>::auto_id(
                    &mut descendents,
                )?),
                "beta8" => Ok(
                    <Join<Beta> as toql::tree::tree_identity::TreeIdentity>::auto_id(
                        &mut descendents,
                    )?,
                ),
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
        mut descendents: &mut I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "beta1" => <Beta as toql::tree::tree_identity::TreeIdentity>::set_id(
                    &mut self.beta1,
                    &mut descendents,
                    action,
                )?,
                "beta2" => <Join<Beta> as toql::tree::tree_identity::TreeIdentity>::set_id(
                    &mut self.beta2,
                    &mut descendents,
                    action,
                )?,
                "beta3" => <Beta as toql::tree::tree_identity::TreeIdentity>::set_id(
                    self.beta3
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta3".to_string()))?,
                    &mut descendents,
                    action,
                )?,
                "beta4" => <Join<Beta> as toql::tree::tree_identity::TreeIdentity>::set_id(
                    self.beta4
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta4".to_string()))?,
                    &mut descendents,
                    action,
                )?,
                "beta5" => <Beta as toql::tree::tree_identity::TreeIdentity>::set_id(
                    self.beta5
                        .as_mut()
                        .unwrap()
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta5".to_string()))?,
                    &mut descendents,
                    action,
                )?,
                "beta6" => <Join<Beta> as toql::tree::tree_identity::TreeIdentity>::set_id(
                    self.beta6
                        .as_mut()
                        .unwrap()
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta6".to_string()))?,
                    &mut descendents,
                    action,
                )?,
                "beta7" => <Beta as toql::tree::tree_identity::TreeIdentity>::set_id(
                    self.beta7
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta7".to_string()))?,
                    &mut descendents,
                    action,
                )?,
                "beta8" => <Join<Beta> as toql::tree::tree_identity::TreeIdentity>::set_id(
                    self.beta8
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta8".to_string()))?,
                    &mut descendents,
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
            None => {}
        }
        Ok(())
    }
}
impl toql::tree::tree_identity::TreeIdentity for &mut Alpha {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: &mut I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)
    }
    #[allow(unused_mut)]
    fn set_id<'a, 'b, I>(
        &mut self,
        mut descendents: &mut I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_identity::TreeIdentity>::set_id(self, descendents, action)
    }
}
impl toql::tree::tree_map::TreeMap for Alpha {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        if registry.get("Alpha").is_none() {
            registry.insert_new_mapper::<Alpha>()?;
        }
        <Beta as toql::tree::tree_map::TreeMap>::map(registry)?;
        <Beta as toql::tree::tree_map::TreeMap>::map(registry)?;
        <Beta as toql::tree::tree_map::TreeMap>::map(registry)?;
        <Beta as toql::tree::tree_map::TreeMap>::map(registry)?;
        <Beta as toql::tree::tree_map::TreeMap>::map(registry)?;
        <Beta as toql::tree::tree_map::TreeMap>::map(registry)?;
        <Beta as toql::tree::tree_map::TreeMap>::map(registry)?;
        <Beta as toql::tree::tree_map::TreeMap>::map(registry)?;
        Ok(())
    }
}
impl toql::tree::tree_map::TreeMap for &Alpha {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        <Alpha as toql::tree::tree_map::TreeMap>::map(registry)
    }
}
impl toql::tree::tree_predicate::TreePredicate for Alpha {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        Ok(match descendents.next() {
            Some(d) => match d.as_str() {
                "beta1" => <Beta as toql::tree::tree_predicate::TreePredicate>::columns(
                    &self.beta1,
                    &mut descendents,
                )?,
                "beta2" => <Join<Beta> as toql::tree::tree_predicate::TreePredicate>::columns(
                    &self.beta2,
                    &mut descendents,
                )?,
                "beta3" => <Beta as toql::tree::tree_predicate::TreePredicate>::columns(
                    self.beta3
                        .as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta3".to_string()))?,
                    &mut descendents,
                )?,
                "beta4" => <Join<Beta> as toql::tree::tree_predicate::TreePredicate>::columns(
                    self.beta4
                        .as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta4".to_string()))?,
                    &mut descendents,
                )?,
                "beta5" => <Beta as toql::tree::tree_predicate::TreePredicate>::columns(
                    self.beta5
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta5".to_string()))?,
                    &mut descendents,
                )?,
                "beta6" => <Join<Beta> as toql::tree::tree_predicate::TreePredicate>::columns(
                    self.beta6
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta6".to_string()))?,
                    &mut descendents,
                )?,
                "beta7" => <Beta as toql::tree::tree_predicate::TreePredicate>::columns(
                    self.beta7
                        .as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta7".to_string()))?,
                    &mut descendents,
                )?,
                "beta8" => <Join<Beta> as toql::tree::tree_predicate::TreePredicate>::columns(
                    self.beta8
                        .as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta8".to_string()))?,
                    &mut descendents,
                )?,
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
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "beta1" => <Beta as toql::tree::tree_predicate::TreePredicate>::args(
                    &self.beta1,
                    &mut descendents,
                    args,
                )?,
                "beta2" => <Join<Beta> as toql::tree::tree_predicate::TreePredicate>::args(
                    &self.beta2,
                    &mut descendents,
                    args,
                )?,
                "beta3" => <Beta as toql::tree::tree_predicate::TreePredicate>::args(
                    self.beta3
                        .as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta3".to_string()))?,
                    &mut descendents,
                    args,
                )?,
                "beta4" => <Join<Beta> as toql::tree::tree_predicate::TreePredicate>::args(
                    self.beta4
                        .as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta4".to_string()))?,
                    &mut descendents,
                    args,
                )?,
                "beta5" => <Beta as toql::tree::tree_predicate::TreePredicate>::args(
                    self.beta5
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta5".to_string()))?,
                    &mut descendents,
                    args,
                )?,
                "beta6" => <Join<Beta> as toql::tree::tree_predicate::TreePredicate>::args(
                    self.beta6
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta6".to_string()))?,
                    &mut descendents,
                    args,
                )?,
                "beta7" => <Beta as toql::tree::tree_predicate::TreePredicate>::args(
                    self.beta7
                        .as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta7".to_string()))?,
                    &mut descendents,
                    args,
                )?,
                "beta8" => <Join<Beta> as toql::tree::tree_predicate::TreePredicate>::args(
                    self.beta8
                        .as_ref()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta8".to_string()))?,
                    &mut descendents,
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
impl toql::tree::tree_predicate::TreePredicate for &Alpha {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_predicate::TreePredicate>::columns(self, descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl toql::tree::tree_predicate::TreePredicate for &mut Alpha {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_predicate::TreePredicate>::columns(self, descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for Alpha
where
    E: std::convert::From<toql::error::ToqlError>,
    AlphaKey: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_index::TreeIndex<R, E>,
{
    #[allow(unused_variables, unused_mut)]
    fn index<'a, I>(
        mut descendents: &mut I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        use toql::from_row::FromRow;
        use toql::sql_builder::select_stream::Select;
        match descendents.next() {
            Some(d) => match d.as_str() {
                "beta1" => <Beta as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    &mut descendents,
                    rows,
                    row_offset,
                    index,
                )?,
                "beta2" => <Join<Beta> as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    &mut descendents,
                    rows,
                    row_offset,
                    index,
                )?,
                "beta3" => <Beta as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    &mut descendents,
                    rows,
                    row_offset,
                    index,
                )?,
                "beta4" => <Join<Beta> as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    &mut descendents,
                    rows,
                    row_offset,
                    index,
                )?,
                "beta5" => <Beta as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    &mut descendents,
                    rows,
                    row_offset,
                    index,
                )?,
                "beta6" => <Join<Beta> as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    &mut descendents,
                    rows,
                    row_offset,
                    index,
                )?,
                "beta7" => <Beta as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    &mut descendents,
                    rows,
                    row_offset,
                    index,
                )?,
                "beta8" => <Join<Beta> as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    &mut descendents,
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
                    let fk = AlphaKey::from_row(&row, &mut i, &mut iter)?.ok_or(
                        toql::error::ToqlError::ValueMissing(
                            <AlphaKey as toql::key::Key>::columns().join(", "),
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
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for &Alpha
where
    E: std::convert::From<toql::error::ToqlError>,
    AlphaKey: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_index::TreeIndex<R, E>,
{
    #[allow(unused_mut)]
    fn index<'a, I>(
        mut descendents: &mut I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_index::TreeIndex<R, E>>::index(
            descendents,
            rows,
            row_offset,
            index,
        )
    }
}
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for Alpha
where
    E: std::convert::From<toql::error::ToqlError>,
    AlphaKey: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
{
    #[allow(unreachable_code, unused_variables, unused_mut)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: &mut I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        use toql::from_row::FromRow;
        use toql::keyed::Keyed;
        use toql::sql_builder::select_stream::Select;
        match descendents.next() {
            Some(d) => match d.as_str() {
                "beta1" => <Beta as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                    &mut self.beta1,
                    &mut descendents,
                    &field,
                    rows,
                    row_offset,
                    index,
                    selection_stream,
                )?,
                "beta2" => <Join<Beta> as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                    &mut self.beta2,
                    &mut descendents,
                    &field,
                    rows,
                    row_offset,
                    index,
                    selection_stream,
                )?,
                "beta3" => <Beta as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                    self.beta3
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta3".to_string()))?,
                    &mut descendents,
                    &field,
                    rows,
                    row_offset,
                    index,
                    selection_stream,
                )?,
                "beta4" => <Join<Beta> as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                    self.beta4
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta4".to_string()))?,
                    &mut descendents,
                    &field,
                    rows,
                    row_offset,
                    index,
                    selection_stream,
                )?,
                "beta5" => <Beta as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                    self.beta5
                        .as_mut()
                        .unwrap()
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta5".to_string()))?,
                    &mut descendents,
                    &field,
                    rows,
                    row_offset,
                    index,
                    selection_stream,
                )?,
                "beta6" => <Join<Beta> as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                    self.beta6
                        .as_mut()
                        .unwrap()
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta6".to_string()))?,
                    &mut descendents,
                    &field,
                    rows,
                    row_offset,
                    index,
                    selection_stream,
                )?,
                "beta7" => <Beta as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                    self.beta7
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta7".to_string()))?,
                    &mut descendents,
                    &field,
                    rows,
                    row_offset,
                    index,
                    selection_stream,
                )?,
                "beta8" => <Join<Beta> as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                    self.beta8
                        .as_mut()
                        .ok_or(toql::error::ToqlError::ValueMissing("beta8".to_string()))?,
                    &mut descendents,
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
                let pk: AlphaKey = <Self as toql::keyed::Keyed>::key(&self);
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
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for &mut Alpha
where
    E: std::convert::From<toql::error::ToqlError>,
    AlphaKey: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
{
    #[allow(unused_mut)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: &mut I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
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

impl<R, E> toql::from_row::FromRow<R, E> for AlphaKey
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
    ) -> std::result::Result<Option<AlphaKey>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        Ok(Some(AlphaKey {
            id: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "AlphaKey::id".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Alpha
where
    E: std::convert::From<toql::error::ToqlError>,
    String: toql::from_row::FromRow<R, E>,
    u64: toql::from_row::FromRow<R, E>,
    Beta: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(
            0 + <u64 as toql::from_row::FromRow<_, E>>::forward(&mut iter)?
                + <String as toql::from_row::FromRow<_, E>>::forward(&mut iter)?
                + if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Beta as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                } else {
                    0
                }
                + if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Beta as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                } else {
                    0
                }
                + if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Beta as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                } else {
                    0
                }
                + if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Beta as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                } else {
                    0
                }
                + if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Beta as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                } else {
                    0
                }
                + if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Beta as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                } else {
                    0
                }
                + if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Beta as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                } else {
                    0
                }
                + if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Beta as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
                } else {
                    0
                },
        )
    }
    #[allow(unused_variables, unused_mut)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Alpha>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        use toql::sql_builder::select_stream::Select;
        Ok(Some(Alpha {
            id: {
                match <u64 as toql::from_row::FromRow<_, E>>::from_row(row, i, iter)? {
                    Some(s) => s,
                    _ => return Ok(None),
                }
            },
            text: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Alpha::text".to_string(),
                    )
                    .into(),
                )?
            },
            beta1: {
                let err = toql::error::ToqlError::DeserializeError(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Alpha::beta1".to_string(),
                    ),
                );
                if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Beta>::from_row(row, i, iter)?.ok_or(err)?
                } else {
                    return Err(err.into());
                }
            },
            beta2: {
                let err = toql::error::ToqlError::DeserializeError(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Alpha::beta2".to_string(),
                    ),
                );
                if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Join<Beta>>::from_row(row, i, iter)?.ok_or(err)?
                } else {
                    return Err(err.into());
                }
            },
            beta3: {
                if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Beta>::from_row(row, i, iter)?
                } else {
                    None
                }
            },
            beta4: {
                if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    <Join<Beta>>::from_row(row, i, iter)?
                } else {
                    None
                }
            },
            beta5: {
                if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    let mut it2 = iter.clone();
                    let n = i.clone();
                    match <Beta>::from_row(row, i, iter)? {
                        Some(f) => Some(Some(f)),
                        None => {
                            let s: usize =
                                <Beta as toql::from_row::FromRow<R, E>>::forward(&mut it2)?;
                            *iter = it2;
                            *i = n + s;
                            Some(None)
                        }
                    }
                } else {
                    None
                }
            },
            beta6: {
                if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    let mut it2 = iter.clone();
                    let n = i.clone();
                    match <Join<Beta>>::from_row(row, i, iter)? {
                        Some(f) => Some(Some(f)),
                        None => {
                            let s: usize =
                                <Join<Beta> as toql::from_row::FromRow<R, E>>::forward(&mut it2)?;
                            *iter = it2;
                            *i = n + s;
                            Some(None)
                        }
                    }
                } else {
                    None
                }
            },
            beta7: {
                if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    return Err(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::SelectionExpected(
                            "Alpha::beta7".to_string(),
                        ),
                    )
                    .into());
                }
                let mut it2 = iter.clone();
                let n = i.clone();
                match <Beta>::from_row(row, i, iter)? {
                    Some(f) => Some(f),
                    None => {
                        let s: usize = <Beta as toql::from_row::FromRow<R, E>>::forward(&mut it2)?;
                        *iter = it2;
                        *i = n + s;
                        None
                    }
                }
            },
            beta8: {
                if iter
                    .next()
                    .ok_or(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::StreamEnd,
                    ))?
                    .is_selected()
                {
                    return Err(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::SelectionExpected(
                            "Alpha::beta8".to_string(),
                        ),
                    )
                    .into());
                }
                let mut it2 = iter.clone();
                let n = i.clone();
                match <Join<Beta>>::from_row(row, i, iter)? {
                    Some(f) => Some(f),
                    None => {
                        let s: usize =
                            <Join<Beta> as toql::from_row::FromRow<R, E>>::forward(&mut it2)?;
                        *iter = it2;
                        *i = n + s;
                        None
                    }
                }
            },
        }))
    }
}

impl toql::query_fields::QueryFields for Alpha {
    type FieldsType = AlphaFields;
    fn fields() -> AlphaFields {
        AlphaFields::new()
    }
    fn fields_from_path(path: String) -> AlphaFields {
        AlphaFields::from_path(path)
    }
}
pub struct AlphaFields(String);
impl toql::query_path::QueryPath for AlphaFields {
    fn into_path(self) -> String {
        self.0
    }
}
impl AlphaFields {
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
    pub fn beta1(mut self) -> <Beta as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("beta1_");
        <Beta as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
    pub fn beta2(mut self) -> <Beta as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("beta2_");
        <Beta as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
    pub fn beta3(mut self) -> <Beta as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("beta3_");
        <Beta as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
    pub fn beta4(mut self) -> <Beta as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("beta4_");
        <Beta as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
    pub fn beta5(mut self) -> <Beta as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("beta5_");
        <Beta as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
    pub fn beta6(mut self) -> <Beta as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("beta6_");
        <Beta as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
    pub fn beta7(mut self) -> <Beta as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("beta7_");
        <Beta as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
    pub fn beta8(mut self) -> <Beta as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("beta8_");
        <Beta as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
}

impl toql::table_mapper::mapped::Mapped for Alpha {
    fn type_name() -> String {
        String::from("Alpha")
    }
    fn table_name() -> String {
        String::from("Alpha")
    }
    fn table_alias() -> String {
        String::from("alpha")
    }
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
        mapper.map_join_with_options(
            "beta1",
            "Beta",
            toql::table_mapper::join_type::JoinType::Inner,
            {
                let mut t = toql::sql_expr::SqlExpr::literal(
                    <Beta as toql::table_mapper::mapped::Mapped>::table_name(),
                );
                t.push_literal(" ");
                t.push_other_alias();
                t
            },
            {
                let mut t = toql::sql_expr::SqlExpr::new();
                <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                    .iter()
                    .for_each(|other_column| {
                        let default_self_column = format!("beta1_{}", other_column);
                        let self_column = {
                            let self_column = match other_column.as_str() {
                                _ => &default_self_column,
                            };
                            self_column
                        };
                        t.push_self_alias();
                        t.push_literal(".");
                        t.push_literal(self_column);
                        t.push_literal(" = ");
                        t.push_other_alias();
                        t.push_literal(".");
                        t.push_literal(other_column);
                        t.push_literal(" AND ");
                    });
                t.pop_literals(5);
                t
            },
            toql::table_mapper::join_options::JoinOptions::new().preselect(true),
        );
        mapper.map_join_with_options(
            "beta2",
            "Beta",
            toql::table_mapper::join_type::JoinType::Inner,
            {
                let mut t = toql::sql_expr::SqlExpr::literal(
                    <Beta as toql::table_mapper::mapped::Mapped>::table_name(),
                );
                t.push_literal(" ");
                t.push_other_alias();
                t
            },
            {
                let mut t = toql::sql_expr::SqlExpr::new();
                <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                    .iter()
                    .for_each(|other_column| {
                        let default_self_column = format!("beta2_{}", other_column);
                        let self_column = {
                            let self_column = match other_column.as_str() {
                                _ => &default_self_column,
                            };
                            self_column
                        };
                        t.push_self_alias();
                        t.push_literal(".");
                        t.push_literal(self_column);
                        t.push_literal(" = ");
                        t.push_other_alias();
                        t.push_literal(".");
                        t.push_literal(other_column);
                        t.push_literal(" AND ");
                    });
                t.pop_literals(5);
                t
            },
            toql::table_mapper::join_options::JoinOptions::new().preselect(true),
        );
        mapper.map_join_with_options(
            "beta3",
            "Beta",
            toql::table_mapper::join_type::JoinType::Inner,
            {
                let mut t = toql::sql_expr::SqlExpr::literal(
                    <Beta as toql::table_mapper::mapped::Mapped>::table_name(),
                );
                t.push_literal(" ");
                t.push_other_alias();
                t
            },
            {
                let mut t = toql::sql_expr::SqlExpr::new();
                <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                    .iter()
                    .for_each(|other_column| {
                        let default_self_column = format!("beta3_{}", other_column);
                        let self_column = {
                            let self_column = match other_column.as_str() {
                                _ => &default_self_column,
                            };
                            self_column
                        };
                        t.push_self_alias();
                        t.push_literal(".");
                        t.push_literal(self_column);
                        t.push_literal(" = ");
                        t.push_other_alias();
                        t.push_literal(".");
                        t.push_literal(other_column);
                        t.push_literal(" AND ");
                    });
                t.pop_literals(5);
                t
            },
            toql::table_mapper::join_options::JoinOptions::new(),
        );
        mapper.map_join_with_options(
            "beta4",
            "Beta",
            toql::table_mapper::join_type::JoinType::Inner,
            {
                let mut t = toql::sql_expr::SqlExpr::literal(
                    <Beta as toql::table_mapper::mapped::Mapped>::table_name(),
                );
                t.push_literal(" ");
                t.push_other_alias();
                t
            },
            {
                let mut t = toql::sql_expr::SqlExpr::new();
                <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                    .iter()
                    .for_each(|other_column| {
                        let default_self_column = format!("beta4_{}", other_column);
                        let self_column = {
                            let self_column = match other_column.as_str() {
                                _ => &default_self_column,
                            };
                            self_column
                        };
                        t.push_self_alias();
                        t.push_literal(".");
                        t.push_literal(self_column);
                        t.push_literal(" = ");
                        t.push_other_alias();
                        t.push_literal(".");
                        t.push_literal(other_column);
                        t.push_literal(" AND ");
                    });
                t.pop_literals(5);
                t
            },
            toql::table_mapper::join_options::JoinOptions::new(),
        );
        mapper.map_join_with_options(
            "beta5",
            "Beta",
            toql::table_mapper::join_type::JoinType::Left,
            {
                let mut t = toql::sql_expr::SqlExpr::literal(
                    <Beta as toql::table_mapper::mapped::Mapped>::table_name(),
                );
                t.push_literal(" ");
                t.push_other_alias();
                t
            },
            {
                let mut t = toql::sql_expr::SqlExpr::new();
                <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                    .iter()
                    .for_each(|other_column| {
                        let default_self_column = format!("beta5_{}", other_column);
                        let self_column = {
                            let self_column = match other_column.as_str() {
                                _ => &default_self_column,
                            };
                            self_column
                        };
                        t.push_self_alias();
                        t.push_literal(".");
                        t.push_literal(self_column);
                        t.push_literal(" = ");
                        t.push_other_alias();
                        t.push_literal(".");
                        t.push_literal(other_column);
                        t.push_literal(" AND ");
                    });
                t.pop_literals(5);
                t
            },
            toql::table_mapper::join_options::JoinOptions::new(),
        );
        mapper.map_join_with_options(
            "beta6",
            "Beta",
            toql::table_mapper::join_type::JoinType::Left,
            {
                let mut t = toql::sql_expr::SqlExpr::literal(
                    <Beta as toql::table_mapper::mapped::Mapped>::table_name(),
                );
                t.push_literal(" ");
                t.push_other_alias();
                t
            },
            {
                let mut t = toql::sql_expr::SqlExpr::new();
                <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                    .iter()
                    .for_each(|other_column| {
                        let default_self_column = format!("beta6_{}", other_column);
                        let self_column = {
                            let self_column = match other_column.as_str() {
                                _ => &default_self_column,
                            };
                            self_column
                        };
                        t.push_self_alias();
                        t.push_literal(".");
                        t.push_literal(self_column);
                        t.push_literal(" = ");
                        t.push_other_alias();
                        t.push_literal(".");
                        t.push_literal(other_column);
                        t.push_literal(" AND ");
                    });
                t.pop_literals(5);
                t
            },
            toql::table_mapper::join_options::JoinOptions::new(),
        );
        mapper.map_join_with_options(
            "beta7",
            "Beta",
            toql::table_mapper::join_type::JoinType::Left,
            {
                let mut t = toql::sql_expr::SqlExpr::literal(
                    <Beta as toql::table_mapper::mapped::Mapped>::table_name(),
                );
                t.push_literal(" ");
                t.push_other_alias();
                t
            },
            {
                let mut t = toql::sql_expr::SqlExpr::new();
                <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                    .iter()
                    .for_each(|other_column| {
                        let default_self_column = format!("beta7_{}", other_column);
                        let self_column = {
                            let self_column = match other_column.as_str() {
                                _ => &default_self_column,
                            };
                            self_column
                        };
                        t.push_self_alias();
                        t.push_literal(".");
                        t.push_literal(self_column);
                        t.push_literal(" = ");
                        t.push_other_alias();
                        t.push_literal(".");
                        t.push_literal(other_column);
                        t.push_literal(" AND ");
                    });
                t.pop_literals(5);
                t
            },
            toql::table_mapper::join_options::JoinOptions::new().preselect(true),
        );
        mapper.map_join_with_options(
            "beta8",
            "Beta",
            toql::table_mapper::join_type::JoinType::Left,
            {
                let mut t = toql::sql_expr::SqlExpr::literal(
                    <Beta as toql::table_mapper::mapped::Mapped>::table_name(),
                );
                t.push_literal(" ");
                t.push_other_alias();
                t
            },
            {
                let mut t = toql::sql_expr::SqlExpr::new();
                <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                    .iter()
                    .for_each(|other_column| {
                        let default_self_column = format!("beta8_{}", other_column);
                        let self_column = {
                            let self_column = match other_column.as_str() {
                                _ => &default_self_column,
                            };
                            self_column
                        };
                        t.push_self_alias();
                        t.push_literal(".");
                        t.push_literal(self_column);
                        t.push_literal(" = ");
                        t.push_other_alias();
                        t.push_literal(".");
                        t.push_literal(other_column);
                        t.push_literal(" AND ");
                    });
                t.pop_literals(5);
                t
            },
            toql::table_mapper::join_options::JoinOptions::new().preselect(true),
        );
        Ok(())
    }
}
impl toql::table_mapper::mapped::Mapped for &Alpha {
    fn type_name() -> String {
        <Alpha as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Alpha as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Alpha as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Alpha as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}
impl toql::table_mapper::mapped::Mapped for &mut Alpha {
    fn type_name() -> String {
        <Alpha as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Alpha as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Alpha as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Alpha as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}

impl toql::tree::tree_insert::TreeInsert for Alpha {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        let mut e = toql::sql_expr::SqlExpr::new();
        match descendents.next() {
            Some(d) => match d.as_str() {
                "beta1" => {
                    return Ok(<Beta as toql::tree::tree_insert::TreeInsert>::columns(
                        &mut descendents,
                    )?);
                }
                "beta2" => {
                    return Ok(
                        <Join<Beta> as toql::tree::tree_insert::TreeInsert>::columns(
                            &mut descendents,
                        )?,
                    );
                }
                "beta3" => {
                    return Ok(<Beta as toql::tree::tree_insert::TreeInsert>::columns(
                        &mut descendents,
                    )?);
                }
                "beta4" => {
                    return Ok(
                        <Join<Beta> as toql::tree::tree_insert::TreeInsert>::columns(
                            &mut descendents,
                        )?,
                    );
                }
                "beta5" => {
                    return Ok(<Beta as toql::tree::tree_insert::TreeInsert>::columns(
                        &mut descendents,
                    )?);
                }
                "beta6" => {
                    return Ok(
                        <Join<Beta> as toql::tree::tree_insert::TreeInsert>::columns(
                            &mut descendents,
                        )?,
                    );
                }
                "beta7" => {
                    return Ok(<Beta as toql::tree::tree_insert::TreeInsert>::columns(
                        &mut descendents,
                    )?);
                }
                "beta8" => {
                    return Ok(
                        <Join<Beta> as toql::tree::tree_insert::TreeInsert>::columns(
                            &mut descendents,
                        )?,
                    );
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
                for other_column in <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("beta1_{}", other_column);
                    let self_column = {
                        let self_column = match other_column.as_str() {
                            _ => &default_self_column,
                        };
                        self_column
                    };
                    e.push_literal(self_column);
                    e.push_literal(", ");
                }
                for other_column in
                    <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("beta2_{}", other_column);
                    let self_column = {
                        let self_column = match other_column.as_str() {
                            _ => &default_self_column,
                        };
                        self_column
                    };
                    e.push_literal(self_column);
                    e.push_literal(", ");
                }
                for other_column in <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("beta3_{}", other_column);
                    let self_column = {
                        let self_column = match other_column.as_str() {
                            _ => &default_self_column,
                        };
                        self_column
                    };
                    e.push_literal(self_column);
                    e.push_literal(", ");
                }
                for other_column in
                    <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("beta4_{}", other_column);
                    let self_column = {
                        let self_column = match other_column.as_str() {
                            _ => &default_self_column,
                        };
                        self_column
                    };
                    e.push_literal(self_column);
                    e.push_literal(", ");
                }
                for other_column in <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("beta5_{}", other_column);
                    let self_column = {
                        let self_column = match other_column.as_str() {
                            _ => &default_self_column,
                        };
                        self_column
                    };
                    e.push_literal(self_column);
                    e.push_literal(", ");
                }
                for other_column in
                    <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("beta6_{}", other_column);
                    let self_column = {
                        let self_column = match other_column.as_str() {
                            _ => &default_self_column,
                        };
                        self_column
                    };
                    e.push_literal(self_column);
                    e.push_literal(", ");
                }
                for other_column in <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("beta7_{}", other_column);
                    let self_column = {
                        let self_column = match other_column.as_str() {
                            _ => &default_self_column,
                        };
                        self_column
                    };
                    e.push_literal(self_column);
                    e.push_literal(", ");
                }
                for other_column in
                    <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("beta8_{}", other_column);
                    let self_column = {
                        let self_column = match other_column.as_str() {
                            _ => &default_self_column,
                        };
                        self_column
                    };
                    e.push_literal(self_column);
                    e.push_literal(", ");
                }
                e.pop_literals(2);
                e.push_literal(")");
            }
        }
        Ok(e)
    }
    #[allow(unused_mut, unused_variables)]
    fn values<'a, I>(
        &self,
        mut descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "beta1" => <Beta as toql::tree::tree_insert::TreeInsert>::values(
                    &self.beta1,
                    &mut descendents,
                    roles,
                    values,
                )?,
                "beta2" => <Join<Beta> as toql::tree::tree_insert::TreeInsert>::values(
                    &self.beta2,
                    &mut descendents,
                    roles,
                    values,
                )?,
                "beta3" => {
                    if let Some(f) = &mut self.beta3.as_ref() {
                        <Beta as toql::tree::tree_insert::TreeInsert>::values(
                            f,
                            &mut descendents,
                            roles,
                            values,
                        )?
                    }
                }
                "beta4" => {
                    if let Some(f) = &mut self.beta4.as_ref() {
                        <Join<Beta> as toql::tree::tree_insert::TreeInsert>::values(
                            f,
                            &mut descendents,
                            roles,
                            values,
                        )?
                    }
                }
                "beta5" => {
                    if let Some(f) = &mut self.beta5.as_ref() {
                        if let Some(f) = f.as_ref() {
                            <Beta as toql::tree::tree_insert::TreeInsert>::values(
                                f,
                                &mut descendents,
                                roles,
                                values,
                            )?
                        }
                    }
                }
                "beta6" => {
                    if let Some(f) = &mut self.beta6.as_ref() {
                        if let Some(f) = f.as_ref() {
                            <Join<Beta> as toql::tree::tree_insert::TreeInsert>::values(
                                f,
                                &mut descendents,
                                roles,
                                values,
                            )?
                        }
                    }
                }
                "beta7" => {
                    if let Some(f) = &mut self.beta7.as_ref() {
                        <Beta as toql::tree::tree_insert::TreeInsert>::values(
                            f,
                            &mut descendents,
                            roles,
                            values,
                        )?
                    }
                }
                "beta8" => {
                    if let Some(f) = &mut self.beta8.as_ref() {
                        <Join<Beta> as toql::tree::tree_insert::TreeInsert>::values(
                            f,
                            &mut descendents,
                            roles,
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
                values.push_literal("(");
                values.push_arg(toql::sql_arg::SqlArg::from(&self.id));
                values.push_literal(", ");
                values.push_arg(toql::sql_arg::SqlArg::from(&self.text));
                values.push_literal(", ");
                &toql::key::Key::params(&<Beta as toql::keyed::Keyed>::key(&self.beta1))
                    .into_iter()
                    .for_each(|a| {
                        values.push_arg(a);
                        values.push_literal(", ");
                    });
                &toql::key::Key::params(&<Join<Beta> as toql::keyed::Keyed>::key(&self.beta2))
                    .into_iter()
                    .for_each(|a| {
                        values.push_arg(a);
                        values.push_literal(", ");
                    });
                if let Some(field) = &self.beta3 {
                    toql::key::Key::params(&<Beta as toql::keyed::Keyed>::key(field))
                        .iter()
                        .for_each(|p| {
                            values.push_arg(p.to_owned());
                            values.push_literal(", ");
                        });
                } else {
                    <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                        .iter()
                        .for_each(|_| {
                            values.push_literal("DEFAULT, ");
                        });
                }
                if let Some(field) = &self.beta4 {
                    toql::key::Key::params(&<Join<Beta> as toql::keyed::Keyed>::key(field))
                        .iter()
                        .for_each(|p| {
                            values.push_arg(p.to_owned());
                            values.push_literal(", ");
                        });
                } else {
                    <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                        .iter()
                        .for_each(|_| {
                            values.push_literal("DEFAULT, ");
                        });
                }
                if let Some(field) = &self.beta5 {
                    if let Some(f) = field {
                        toql::key::Key::params(&<Beta as toql::keyed::Keyed>::key(f))
                            .iter()
                            .for_each(|p| {
                                values.push_arg(p.to_owned());
                                values.push_literal(", ");
                            });
                    } else {
                        <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            .iter()
                            .for_each(|_| {
                                values.push_arg(toql::sql_arg::SqlArg::Null);
                                values.push_literal(", ");
                            });
                    }
                } else {
                    <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                        .iter()
                        .for_each(|_| {
                            values.push_literal("DEFAULT, ");
                        });
                }
                if let Some(field) = &self.beta6 {
                    if let Some(f) = field {
                        toql::key::Key::params(&<Join<Beta> as toql::keyed::Keyed>::key(f))
                            .iter()
                            .for_each(|p| {
                                values.push_arg(p.to_owned());
                                values.push_literal(", ");
                            });
                    } else {
                        <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            .iter()
                            .for_each(|_| {
                                values.push_arg(toql::sql_arg::SqlArg::Null);
                                values.push_literal(", ");
                            });
                    }
                } else {
                    <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                        .iter()
                        .for_each(|_| {
                            values.push_literal("DEFAULT, ");
                        });
                }
                if let Some(f) = &self.beta7 {
                    toql::key::Key::params(&<Beta as toql::keyed::Keyed>::key(f))
                        .iter()
                        .for_each(|p| {
                            values.push_arg(p.to_owned());
                            values.push_literal(", ");
                        });
                } else {
                    <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                        .iter()
                        .for_each(|_| {
                            values.push_arg(toql::sql_arg::SqlArg::Null);
                            values.push_literal(", ");
                        });
                }
                if let Some(f) = &self.beta8 {
                    toql::key::Key::params(&<Join<Beta> as toql::keyed::Keyed>::key(f))
                        .iter()
                        .for_each(|p| {
                            values.push_arg(p.to_owned());
                            values.push_literal(", ");
                        });
                } else {
                    <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                        .iter()
                        .for_each(|_| {
                            values.push_arg(toql::sql_arg::SqlArg::Null);
                            values.push_literal(", ");
                        });
                }
                values.pop_literals(2);
                values.push_literal("), ");
            }
        }
        Ok(())
    }
}
impl toql::tree::tree_insert::TreeInsert for &Alpha {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, I>(
        &self,
        mut descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, values)
    }
}
impl toql::tree::tree_insert::TreeInsert for &mut Alpha {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, I>(
        &self,
        mut descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, values)
    }
}

impl toql::tree::tree_update::TreeUpdate for Alpha {
    #[allow(unused_mut, unused_variables, unused_parens)]
    fn update<'a, I>(
        &self,
        mut descendents: &mut I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "beta1" => <Beta as toql::tree::tree_update::TreeUpdate>::update(
                    &self.beta1,
                    &mut descendents,
                    fields,
                    roles,
                    exprs,
                )?,
                "beta2" => <Join<Beta> as toql::tree::tree_update::TreeUpdate>::update(
                    &self.beta2,
                    &mut descendents,
                    fields,
                    roles,
                    exprs,
                )?,
                "beta3" => {
                    if let Some(f) = self.beta3.as_ref() {
                        <Beta as toql::tree::tree_update::TreeUpdate>::update(
                            f,
                            &mut descendents,
                            fields,
                            roles,
                            exprs,
                        )?
                    }
                }
                "beta4" => {
                    if let Some(f) = self.beta4.as_ref() {
                        <Join<Beta> as toql::tree::tree_update::TreeUpdate>::update(
                            f,
                            &mut descendents,
                            fields,
                            roles,
                            exprs,
                        )?
                    }
                }
                "beta5" => {
                    if let Some(f1) = self.beta5.as_ref() {
                        if let Some(f2) = f1 {
                            <Beta as toql::tree::tree_update::TreeUpdate>::update(
                                f2,
                                &mut descendents,
                                fields,
                                roles,
                                exprs,
                            )?
                        }
                    }
                }
                "beta6" => {
                    if let Some(f1) = self.beta6.as_ref() {
                        if let Some(f2) = f1 {
                            <Join<Beta> as toql::tree::tree_update::TreeUpdate>::update(
                                f2,
                                &mut descendents,
                                fields,
                                roles,
                                exprs,
                            )?
                        }
                    }
                }
                "beta7" => {
                    if let Some(f) = self.beta7.as_ref() {
                        <Beta as toql::tree::tree_update::TreeUpdate>::update(
                            f,
                            &mut descendents,
                            fields,
                            roles,
                            exprs,
                        )?
                    }
                }
                "beta8" => {
                    if let Some(f) = self.beta8.as_ref() {
                        <Join<Beta> as toql::tree::tree_update::TreeUpdate>::update(
                            f,
                            &mut descendents,
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
                let path_selected = fields.contains("*");
                let mut expr = toql::sql_expr::SqlExpr::new();
                expr.push_literal("UPDATE ");
                expr.push_literal("Alpha");
                expr.push_literal(" ");
                expr.push_literal(" SET ");
                let tokens = expr.tokens().len();
                if (path_selected || fields.contains("text")) {
                    expr.push_literal("text");
                    expr.push_literal(" = ");
                    expr.push_arg(toql::sql_arg::SqlArg::from(&self.text));
                    expr.push_literal(", ");
                }
                if (path_selected || fields.contains("beta1")) {
                    let inverse_columns =
                        <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            .iter()
                            .enumerate()
                            .map(|(i, c)| format!("beta1_{}", c))
                            .collect::<Vec<String>>();
                    let args = toql::key::Key::params(&toql::keyed::Keyed::key(&self.beta1));
                    for (c, a) in inverse_columns.iter().zip(args) {
                        expr.push_literal(c);
                        expr.push_literal(" = ");
                        expr.push_arg(a);
                        expr.push_literal(", ");
                    }
                }
                if (path_selected || fields.contains("beta2")) {
                    let inverse_columns =
                        <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            .iter()
                            .enumerate()
                            .map(|(i, c)| format!("beta2_{}", c))
                            .collect::<Vec<String>>();
                    let args = toql::key::Key::params(&toql::keyed::Keyed::key(&self.beta2));
                    for (c, a) in inverse_columns.iter().zip(args) {
                        expr.push_literal(c);
                        expr.push_literal(" = ");
                        expr.push_arg(a);
                        expr.push_literal(", ");
                    }
                }
                if self.beta3.is_some() && (path_selected || fields.contains("beta3")) {
                    let inverse_columns =
                        <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            .iter()
                            .enumerate()
                            .map(|(i, c)| format!("beta3_{}", c))
                            .collect::<Vec<String>>();
                    let args = toql::key::Key::params(&toql::keyed::Keyed::key(
                        &self.beta3.as_ref().unwrap(),
                    ));
                    for (c, a) in inverse_columns.iter().zip(args) {
                        expr.push_literal(c);
                        expr.push_literal(" = ");
                        expr.push_arg(a);
                        expr.push_literal(", ");
                    }
                }
                if self.beta4.is_some() && (path_selected || fields.contains("beta4")) {
                    let inverse_columns =
                        <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            .iter()
                            .enumerate()
                            .map(|(i, c)| format!("beta4_{}", c))
                            .collect::<Vec<String>>();
                    let args = toql::key::Key::params(&toql::keyed::Keyed::key(
                        &self.beta4.as_ref().unwrap(),
                    ));
                    for (c, a) in inverse_columns.iter().zip(args) {
                        expr.push_literal(c);
                        expr.push_literal(" = ");
                        expr.push_arg(a);
                        expr.push_literal(", ");
                    }
                }
                if self.beta5.is_some() && (path_selected || fields.contains("beta5")) {
                    let inverse_columns =
                        <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            .iter()
                            .enumerate()
                            .map(|(i, c)| format!("beta5_{}", c))
                            .collect::<Vec<String>>();
                    let args = if let Some(entity) = self.beta5.as_ref().unwrap() {
                        toql::key::Key::params(&toql::keyed::Keyed::key(&entity))
                    } else {
                        inverse_columns
                            .iter()
                            .map(|c| toql::sql_arg::SqlArg::Null)
                            .collect::<Vec<_>>()
                    };
                    for (c, a) in inverse_columns.iter().zip(args) {
                        expr.push_literal(c);
                        expr.push_literal(" = ");
                        expr.push_arg(a);
                        expr.push_literal(", ");
                    }
                }
                if self.beta6.is_some() && (path_selected || fields.contains("beta6")) {
                    let inverse_columns =
                        <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            .iter()
                            .enumerate()
                            .map(|(i, c)| format!("beta6_{}", c))
                            .collect::<Vec<String>>();
                    let args = if let Some(entity) = self.beta6.as_ref().unwrap() {
                        toql::key::Key::params(&toql::keyed::Keyed::key(&entity))
                    } else {
                        inverse_columns
                            .iter()
                            .map(|c| toql::sql_arg::SqlArg::Null)
                            .collect::<Vec<_>>()
                    };
                    for (c, a) in inverse_columns.iter().zip(args) {
                        expr.push_literal(c);
                        expr.push_literal(" = ");
                        expr.push_arg(a);
                        expr.push_literal(", ");
                    }
                }
                if (path_selected || fields.contains("beta7")) {
                    let inverse_columns =
                        <<Beta as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            .iter()
                            .enumerate()
                            .map(|(i, c)| format!("beta7_{}", c))
                            .collect::<Vec<String>>();
                    let args = toql::key::Key::params(&toql::keyed::Keyed::key(
                        &self.beta7.as_ref().unwrap(),
                    ));
                    for (c, a) in inverse_columns.iter().zip(args) {
                        expr.push_literal(c);
                        expr.push_literal(" = ");
                        expr.push_arg(a);
                        expr.push_literal(", ");
                    }
                }
                if (path_selected || fields.contains("beta8")) {
                    let inverse_columns =
                        <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            .iter()
                            .enumerate()
                            .map(|(i, c)| format!("beta8_{}", c))
                            .collect::<Vec<String>>();
                    let args = toql::key::Key::params(&toql::keyed::Keyed::key(
                        &self.beta8.as_ref().unwrap(),
                    ));
                    for (c, a) in inverse_columns.iter().zip(args) {
                        expr.push_literal(c);
                        expr.push_literal(" = ");
                        expr.push_arg(a);
                        expr.push_literal(", ");
                    }
                }
                expr.pop();
                if expr.tokens().len() > tokens {
                    expr.push_literal(" WHERE ");
                    let key = <Self as toql::keyed::Keyed>::key(&self);
                    let resolver =
                        toql::sql_expr::resolver::Resolver::new().with_self_alias("Alpha");
                    expr.extend(resolver.alias_to_literals(&toql::key::Key::predicate_expr(&key))?);
                    exprs.push(expr);
                }
            }
        };
        Ok(())
    }
}
impl toql::tree::tree_update::TreeUpdate for &mut Alpha {
    #[allow(unused_mut)]
    fn update<'a, I>(
        &self,
        mut descendents: &mut I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha as toql::tree::tree_update::TreeUpdate>::update(
            self,
            descendents,
            fields,
            roles,
            exprs,
        )
    }
}

impl<R, E> toql::backend::Load<R, E> for Alpha2
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>
        + std::fmt::Debug,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl<R, E> toql::backend::Load<R, E> for &Alpha2
where
    Self: toql::keyed::Keyed
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::from_row::FromRow<R, E>
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_index::TreeIndex<R, E>
        + toql::tree::tree_merge::TreeMerge<R, E>
        + std::fmt::Debug,
    <Self as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
    E: std::convert::From<toql::error::ToqlError>,
{
}
impl toql::backend::Insert for Alpha2 where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::Insert for &mut Alpha2 where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::Update for Alpha2 where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::Update for &mut Alpha2 where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::table_mapper::mapped::Mapped
        + toql::tree::tree_map::TreeMap
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::Count for Alpha2 where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::Count for &Alpha2 where
    Self: toql::keyed::Keyed + toql::table_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::Delete for Alpha2 where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
{
}
impl toql::backend::Delete for &Alpha2 where
    Self: toql::table_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
{
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Alpha2Key {
    pub id: <Join<Beta> as toql::keyed::Keyed>::Key,
}
impl toql::key_fields::KeyFields for Alpha2Key {
    type Entity = Alpha2;
    fn fields() -> Vec<String> {
        let mut fields: Vec<String> = Vec::new();
        <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key_fields::KeyFields>::fields()
            .iter()
            .for_each(|other_field| {
                fields.push(format!("{}_{}", "id", other_field));
            });
        fields
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.extend_from_slice(&toql::key::Key::params(&key.id));
        params
    }
}
impl toql::key_fields::KeyFields for &Alpha2Key {
    type Entity = Alpha2;
    fn fields() -> Vec<String> {
        <Alpha2Key as toql::key_fields::KeyFields>::fields()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <Alpha2Key as toql::key_fields::KeyFields>::params(self)
    }
}
impl toql::key::Key for Alpha2Key {
    type Entity = Alpha2;
    fn columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
            .iter()
            .for_each(|other_column| {
                let default_self_column = format!("id_{}", other_column);
                let column = {
                    let self_column = match other_column.as_str() {
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
        <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::default_inverse_columns()
            .iter()
            .for_each(|other_column| {
                let default_self_column = format!("id_{}", other_column);
                let column = {
                    let self_column = match other_column.as_str() {
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
        params.extend_from_slice(&toql::key::Key::params(&key.id));
        params
    }
}
impl toql::key::Key for &Alpha2Key {
    type Entity = Alpha2;
    fn columns() -> Vec<String> {
        <Alpha2Key as toql::key::Key>::columns()
    }
    fn default_inverse_columns() -> Vec<String> {
        <Alpha2Key as toql::key::Key>::default_inverse_columns()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <Alpha2Key as toql::key::Key>::params(self)
    }
}
impl toql::keyed::Keyed for Alpha2 {
    type Key = Alpha2Key;
    fn key(&self) -> Self::Key {
        Alpha2Key {
            id: <Join<Beta> as toql::keyed::Keyed>::key(&self.id),
        }
    }
}
impl toql::keyed::Keyed for &Alpha2 {
    type Key = Alpha2Key;
    fn key(&self) -> Self::Key {
        <Alpha2 as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::Keyed for &mut Alpha2 {
    type Key = Alpha2Key;
    fn key(&self) -> Self::Key {
        <Alpha2 as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::KeyedMut for Alpha2 {
    fn set_key(&mut self, key: Self::Key) {
        <Join<Beta> as toql::keyed::KeyedMut>::set_key(&mut self.id, key.id);
    }
}
impl toql::keyed::KeyedMut for &mut Alpha2 {
    fn set_key(&mut self, key: Self::Key) {
        <Alpha2 as toql::keyed::KeyedMut>::set_key(self, key)
    }
}
impl std::convert::TryFrom<Vec<toql::sql_arg::SqlArg>> for Alpha2Key {
    type Error = toql::error::ToqlError;
    fn try_from(args: Vec<toql::sql_arg::SqlArg>) -> toql::result::Result<Self> {
        use std::convert::TryInto;
        Ok(Alpha2Key {
            id: <Join<Beta> as toql::keyed::Keyed>::Key::try_from(Vec::from(&args[0..]))?,
        })
    }
}
impl std::convert::From<<Join<Beta> as toql::keyed::Keyed>::Key> for Alpha2Key {
    fn from(key: <Join<Beta> as toql::keyed::Keyed>::Key) -> Self {
        Self { id: key }
    }
}
impl std::hash::Hash for Alpha2 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        <Alpha2 as toql::keyed::Keyed>::key(self).hash(state);
    }
}

impl toql::tree::tree_identity::TreeIdentity for Alpha2 {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: &mut I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => Ok(
                    <Join<Beta> as toql::tree::tree_identity::TreeIdentity>::auto_id(
                        &mut descendents,
                    )?,
                ),
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
        mut descendents: &mut I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => <Join<Beta> as toql::tree::tree_identity::TreeIdentity>::set_id(
                    &mut self.id,
                    &mut descendents,
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
            None => {}
        }
        Ok(())
    }
}
impl toql::tree::tree_identity::TreeIdentity for &mut Alpha2 {
    #[allow(unused_mut)]
    fn auto_id<'a, I>(mut descendents: &mut I) -> std::result::Result<bool, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_identity::TreeIdentity>::auto_id(descendents)
    }
    #[allow(unused_mut)]
    fn set_id<'a, 'b, I>(
        &mut self,
        mut descendents: &mut I,
        action: &'b toql::tree::tree_identity::IdentityAction,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_identity::TreeIdentity>::set_id(self, descendents, action)
    }
}
impl toql::tree::tree_map::TreeMap for Alpha2 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        if registry.get("Alpha2").is_none() {
            registry.insert_new_mapper::<Alpha2>()?;
        }
        <Beta as toql::tree::tree_map::TreeMap>::map(registry)?;
        Ok(())
    }
}
impl toql::tree::tree_map::TreeMap for &Alpha2 {
    fn map(
        registry: &mut toql::table_mapper_registry::TableMapperRegistry,
    ) -> toql::result::Result<()> {
        <Alpha2 as toql::tree::tree_map::TreeMap>::map(registry)
    }
}
impl toql::tree::tree_predicate::TreePredicate for Alpha2 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        Ok(match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => <Join<Beta> as toql::tree::tree_predicate::TreePredicate>::columns(
                    &self.id,
                    &mut descendents,
                )?,
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
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => <Join<Beta> as toql::tree::tree_predicate::TreePredicate>::args(
                    &self.id,
                    &mut descendents,
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
impl toql::tree::tree_predicate::TreePredicate for &Alpha2 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_predicate::TreePredicate>::columns(self, descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl toql::tree::tree_predicate::TreePredicate for &mut Alpha2 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_predicate::TreePredicate>::columns(self, descendents)
    }
    #[allow(unused_mut)]
    fn args<'a, I>(
        &self,
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for Alpha2
where
    E: std::convert::From<toql::error::ToqlError>,
    Alpha2Key: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_index::TreeIndex<R, E>,
{
    #[allow(unused_variables, unused_mut)]
    fn index<'a, I>(
        mut descendents: &mut I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        use toql::from_row::FromRow;
        use toql::sql_builder::select_stream::Select;
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => <Join<Beta> as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    &mut descendents,
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
                    let fk = Alpha2Key::from_row(&row, &mut i, &mut iter)?.ok_or(
                        toql::error::ToqlError::ValueMissing(
                            <Alpha2Key as toql::key::Key>::columns().join(", "),
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
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for &Alpha2
where
    E: std::convert::From<toql::error::ToqlError>,
    Alpha2Key: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_index::TreeIndex<R, E>,
{
    #[allow(unused_mut)]
    fn index<'a, I>(
        mut descendents: &mut I,
        rows: &[R],
        row_offset: usize,
        index: &mut std::collections::HashMap<u64, Vec<usize>>,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_index::TreeIndex<R, E>>::index(
            descendents,
            rows,
            row_offset,
            index,
        )
    }
}
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for Alpha2
where
    E: std::convert::From<toql::error::ToqlError>,
    Alpha2Key: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
{
    #[allow(unreachable_code, unused_variables, unused_mut)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: &mut I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        use toql::from_row::FromRow;
        use toql::keyed::Keyed;
        use toql::sql_builder::select_stream::Select;
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => <Join<Beta> as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                    &mut self.id,
                    &mut descendents,
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
                let pk: Alpha2Key = <Self as toql::keyed::Keyed>::key(&self);
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
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for &mut Alpha2
where
    E: std::convert::From<toql::error::ToqlError>,
    Alpha2Key: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
{
    #[allow(unused_mut)]
    fn merge<'a, I>(
        &mut self,
        mut descendents: &mut I,
        field: &str,
        rows: &[R],
        row_offset: usize,
        index: &std::collections::HashMap<u64, Vec<usize>>,
        selection_stream: &toql::sql_builder::select_stream::SelectStream,
    ) -> std::result::Result<(), E>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
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

impl<R, E> toql::from_row::FromRow<R, E> for Alpha2Key
where
    E: std::convert::From<toql::error::ToqlError>,
    Beta: toql::from_row::FromRow<R, E> + toql::keyed::Keyed,
    <Beta as toql::keyed::Keyed>::Key: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(0 + <Beta as toql::from_row::FromRow<R, E>>::forward(&mut iter)?)
    }
    #[allow(unused_variables, unused_mut)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Alpha2Key>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        Ok(Some(Alpha2Key {
            id: {
                <<Join<Beta> as toql::keyed::Keyed>::Key>::from_row(row, i, iter)?.ok_or(
                    toql::error::ToqlError::ValueMissing("Alpha2Key::id".to_string()),
                )?
            },
        }))
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Alpha2
where
    E: std::convert::From<toql::error::ToqlError>,
    Beta: toql::from_row::FromRow<R, E>,
    String: toql::from_row::FromRow<R, E>,
{
    fn forward<'a, I>(mut iter: &mut I) -> std::result::Result<usize, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select>,
    {
        Ok(0 + if iter
            .next()
            .ok_or(toql::error::ToqlError::DeserializeError(
                toql::deserialize::error::DeserializeError::StreamEnd,
            ))?
            .is_selected()
        {
            <Beta as toql::from_row::FromRow<R, E>>::forward(&mut iter)?
        } else {
            0
        } + <String as toql::from_row::FromRow<_, E>>::forward(&mut iter)?)
    }
    #[allow(unused_variables, unused_mut)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Alpha2>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        use toql::sql_builder::select_stream::Select;
        Ok(Some(Alpha2 {
            id: {
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
                                toql::error::ToqlError::ValueMissing("id".to_string()).into()
                            )
                        }
                    }
                } else {
                    return Err(toql::error::ToqlError::DeserializeError(
                        toql::deserialize::error::DeserializeError::SelectionExpected(
                            "id".to_string(),
                        ),
                    )
                    .into());
                }
            },
            text: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Alpha2::text".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl toql::query_fields::QueryFields for Alpha2 {
    type FieldsType = Alpha2Fields;
    fn fields() -> Alpha2Fields {
        Alpha2Fields::new()
    }
    fn fields_from_path(path: String) -> Alpha2Fields {
        Alpha2Fields::from_path(path)
    }
}
pub struct Alpha2Fields(String);
impl toql::query_path::QueryPath for Alpha2Fields {
    fn into_path(self) -> String {
        self.0
    }
}
impl Alpha2Fields {
    pub fn new() -> Self {
        Self::from_path(String::from(""))
    }
    pub fn from_path(path: String) -> Self {
        Self(path)
    }
    pub fn into_name(self) -> String {
        self.0
    }
    pub fn id(mut self) -> <Beta as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("id_");
        <Beta as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
    pub fn text(mut self) -> toql::query::field::Field {
        self.0.push_str("text");
        toql::query::field::Field::from(self.0)
    }
}

impl toql::table_mapper::mapped::Mapped for Alpha2 {
    fn type_name() -> String {
        String::from("Alpha2")
    }
    fn table_name() -> String {
        String::from("Alpha2")
    }
    fn table_alias() -> String {
        String::from("alpha2")
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        mapper.map_join_with_options(
            "id",
            "Beta",
            toql::table_mapper::join_type::JoinType::Inner,
            {
                let mut t = toql::sql_expr::SqlExpr::literal(
                    <Beta as toql::table_mapper::mapped::Mapped>::table_name(),
                );
                t.push_literal(" ");
                t.push_other_alias();
                t
            },
            {
                let mut t = toql::sql_expr::SqlExpr::new();
                <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                    .iter()
                    .for_each(|other_column| {
                        let default_self_column = format!("id_{}", other_column);
                        let self_column = {
                            let self_column = match other_column.as_str() {
                                _ => &default_self_column,
                            };
                            self_column
                        };
                        t.push_self_alias();
                        t.push_literal(".");
                        t.push_literal(self_column);
                        t.push_literal(" = ");
                        t.push_other_alias();
                        t.push_literal(".");
                        t.push_literal(other_column);
                        t.push_literal(" AND ");
                    });
                t.pop_literals(5);
                t
            },
            toql::table_mapper::join_options::JoinOptions::new()
                .preselect(true)
                .key(true),
        );
        mapper.map_column_with_options(
            "text",
            "text",
            toql::table_mapper::field_options::FieldOptions::new().preselect(true),
        );
        Ok(())
    }
}
impl toql::table_mapper::mapped::Mapped for &Alpha2 {
    fn type_name() -> String {
        <Alpha2 as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Alpha2 as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Alpha2 as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Alpha2 as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}
impl toql::table_mapper::mapped::Mapped for &mut Alpha2 {
    fn type_name() -> String {
        <Alpha2 as toql::table_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Alpha2 as toql::table_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Alpha2 as toql::table_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::table_mapper::TableMapper) -> toql::result::Result<()> {
        <Alpha2 as toql::table_mapper::mapped::Mapped>::map(mapper)
    }
}

impl toql::tree::tree_insert::TreeInsert for Alpha2 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        let mut e = toql::sql_expr::SqlExpr::new();
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => {
                    return Ok(
                        <Join<Beta> as toql::tree::tree_insert::TreeInsert>::columns(
                            &mut descendents,
                        )?,
                    );
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
                for other_column in
                    <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("id_{}", other_column);
                    let self_column = {
                        let self_column = match other_column.as_str() {
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
    fn values<'a, I>(
        &self,
        mut descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "id" => <Join<Beta> as toql::tree::tree_insert::TreeInsert>::values(
                    &self.id,
                    &mut descendents,
                    roles,
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
                values.push_literal("(");
                &toql::key::Key::params(&<Join<Beta> as toql::keyed::Keyed>::key(&self.id))
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
impl toql::tree::tree_insert::TreeInsert for &Alpha2 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, I>(
        &self,
        mut descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, values)
    }
}
impl toql::tree::tree_insert::TreeInsert for &mut Alpha2 {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_insert::TreeInsert>::columns(descendents)
    }
    #[allow(unused_mut)]
    fn values<'a, I>(
        &self,
        mut descendents: &mut I,
        roles: &std::collections::HashSet<String>,
        values: &mut toql::sql_expr::SqlExpr,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, values)
    }
}

impl toql::tree::tree_update::TreeUpdate for Alpha2 {
    #[allow(unused_mut, unused_variables, unused_parens)]
    fn update<'a, I>(
        &self,
        mut descendents: &mut I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
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
                let path_selected = fields.contains("*");
                let mut expr = toql::sql_expr::SqlExpr::new();
                expr.push_literal("UPDATE ");
                expr.push_literal("Alpha2");
                expr.push_literal(" ");
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
                        toql::sql_expr::resolver::Resolver::new().with_self_alias("Alpha2");
                    expr.extend(resolver.alias_to_literals(&toql::key::Key::predicate_expr(&key))?);
                    exprs.push(expr);
                }
            }
        };
        Ok(())
    }
}
impl toql::tree::tree_update::TreeUpdate for &mut Alpha2 {
    #[allow(unused_mut)]
    fn update<'a, I>(
        &self,
        mut descendents: &mut I,
        fields: &std::collections::HashSet<String>,
        roles: &std::collections::HashSet<String>,
        exprs: &mut Vec<toql::sql_expr::SqlExpr>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Alpha2 as toql::tree::tree_update::TreeUpdate>::update(
            self,
            descendents,
            fields,
            roles,
            exprs,
        )
    }
}
 */