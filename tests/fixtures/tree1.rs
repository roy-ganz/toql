use toql::prelude::{Join, Toql};

/*
#[derive(Debug, PartialEq, Toql)]
pub struct Alpha {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(join())]
    beta: Join<Beta>,

    #[toql(merge())]
    gamma: Vec<Gamma>
}

#[derive(Debug, PartialEq, Toql)]
pub struct Beta {
    #[toql(key)]
    id: u64,
    text: String,
}

#[derive(Debug, PartialEq, Toql)]
pub struct Gamma {
    #[toql(key)]
    id: u64,
    text: String
}
*/
impl Default for Alpha {
    fn default() -> Alpha {
        Alpha {
            id: 1,
            text: "Alpha".to_string(),
            beta: Join::Entity(Box::new(Beta {
                id: 11,
                text: "Beta".to_string(),
            })),
            gamma: vec![
                Gamma {
                    id: 12,
                    text: "Gamma1".to_string(),
                },
                Gamma {
                    id: 13,
                    text: "Gamma2".to_string(),
                },
            ],
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Alpha {
    id: u64,
    text: String,

    beta: Join<Beta>,

    gamma: Vec<Gamma>,
}

#[derive(Debug, PartialEq)]
pub struct Beta {
    id: u64,
    text: String,
}

#[derive(Debug, PartialEq)]
pub struct Gamma {
    id: u64,
    text: String,
}

impl<R, E> toql::backend::api::Load<R, E> for Alpha
where
    Self: toql::keyed::Keyed
        + toql::sql_mapper::mapped::Mapped
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
impl<R, E> toql::backend::api::Load<R, E> for &Alpha
where
    Self: toql::keyed::Keyed
        + toql::sql_mapper::mapped::Mapped
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
impl toql::backend::api::Insert for Alpha where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::sql_mapper::mapped::Mapped
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::api::Insert for &mut Alpha where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::sql_mapper::mapped::Mapped
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::api::Update for Alpha where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::sql_mapper::mapped::Mapped
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::api::Update for &mut Alpha where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::sql_mapper::mapped::Mapped
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::api::Count for Alpha where
    Self: toql::keyed::Keyed + toql::sql_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::api::Count for &Alpha where
    Self: toql::keyed::Keyed + toql::sql_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::api::Delete for Alpha where
    Self: toql::sql_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
{
}
impl toql::backend::api::Delete for &Alpha where
    Self: toql::sql_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
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
    fn auto_id() -> bool {
        false
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
                "beta" => <Join<Beta> as toql::tree::tree_identity::TreeIdentity>::set_id(
                    &mut self.beta,
                    &mut descendents,
                    action,
                )?,
                "gamma" => {
                    for f in &mut self.gamma {
                        <Gamma as toql::tree::tree_identity::TreeIdentity>::set_id(
                            f,
                            &mut descendents,
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
            None => {}
        }
        Ok(())
    }
}
impl toql::tree::tree_identity::TreeIdentity for &mut Alpha {
    fn auto_id() -> bool {
        <Alpha as toql::tree::tree_identity::TreeIdentity>::auto_id()
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
        registry: &mut toql::sql_mapper_registry::SqlMapperRegistry,
    ) -> toql::result::Result<()> {
        if registry.get("Alpha").is_none() {
            registry.insert_new_mapper::<Alpha>()?;
        }
        <Beta as toql::tree::tree_map::TreeMap>::map(registry)?;
        <Gamma as toql::tree::tree_map::TreeMap>::map(registry)?;
        Ok(())
    }
}
impl toql::tree::tree_map::TreeMap for &Alpha {
    fn map(
        registry: &mut toql::sql_mapper_registry::SqlMapperRegistry,
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
                "beta" => <Join<Beta> as toql::tree::tree_predicate::TreePredicate>::columns(
                    &self.beta,
                    &mut descendents,
                )?,
                "gamma" => {
                    let f = &self
                        .gamma
                        .get(0)
                        .ok_or(toql::error::ToqlError::SqlBuilderError(
                            toql::sql_builder::sql_builder_error::SqlBuilderError::FieldMissing(
                                "gamma".to_string(),
                            ),
                        ))?;
                    <Gamma as toql::tree::tree_predicate::TreePredicate>::columns(
                        f,
                        &mut descendents,
                    )?
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
        mut descendents: &mut I,
        args: &mut Vec<toql::sql_arg::SqlArg>,
    ) -> std::result::Result<(), toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        match descendents.next() {
            Some(d) => match d.as_str() {
                "beta" => <Join<Beta> as toql::tree::tree_predicate::TreePredicate>::args(
                    &self.beta,
                    &mut descendents,
                    args,
                )?,
                "gamma" => {
                    for f in &self.gamma {
                        <Gamma as toql::tree::tree_predicate::TreePredicate>::args(
                            f,
                            &mut descendents,
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
    Gamma: toql::tree::tree_index::TreeIndex<R, E>,
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
                "beta" => <Join<Beta> as toql::tree::tree_index::TreeIndex<R, E>>::index(
                    &mut descendents,
                    rows,
                    row_offset,
                    index,
                )?,
                "gamma" => <Gamma as toql::tree::tree_index::TreeIndex<R, E>>::index(
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
    Gamma: toql::tree::tree_index::TreeIndex<R, E>,
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
    Gamma: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
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
                "beta" => <Join<Beta> as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                    &mut self.beta,
                    &mut descendents,
                    &field,
                    rows,
                    row_offset,
                    index,
                    selection_stream,
                )?,
                "gamma" => {
                    for f in &mut self.gamma {
                        <Gamma as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
                            f,
                            &mut descendents,
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
                let pk: AlphaKey = <Self as toql::keyed::Keyed>::key(&self);
                let mut s = DefaultHasher::new();
                pk.hash(&mut s);
                let h = s.finish();
                let default_vec: Vec<usize> = Vec::new();
                let row_numbers: &Vec<usize> = index.get(&h).unwrap_or(&default_vec);
                let n = row_offset;
                match field {
                    "gamma" => {
                        for row_number in row_numbers {
                            let mut i = n;
                            let mut iter = std::iter::repeat(&Select::Query);
                            let row: &R = &rows[*row_number];
                            let fk = AlphaKey::from_row(&row, &mut i, &mut iter)?
                                .ok_or(toql::error::ToqlError::ValueMissing("gamma".to_string()))?;
                            if fk == pk {
                                let mut iter = selection_stream.iter();
                                let e = Gamma::from_row(&row, &mut i, &mut iter)?.ok_or(
                                    toql::error::ToqlError::ValueMissing("gamma".to_string()),
                                )?;
                                self.gamma.push(e);
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
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for &mut Alpha
where
    E: std::convert::From<toql::error::ToqlError>,
    AlphaKey: toql::from_row::FromRow<R, E>,
    Beta: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
    Gamma: toql::tree::tree_merge::TreeMerge<R, E> + toql::from_row::FromRow<R, E>,
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
    u64: toql::from_row::FromRow<R, E>,
    Beta: toql::from_row::FromRow<R, E>,
    String: toql::from_row::FromRow<R, E>,
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
            beta: {
                let err = toql::error::ToqlError::DeserializeError(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Alpha::beta".to_string(),
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
            gamma: Vec::new(),
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
    pub fn beta(mut self) -> <Beta as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("beta_");
        <Beta as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
    pub fn gamma(mut self) -> <Gamma as toql::query_fields::QueryFields>::FieldsType {
        self.0.push_str("gamma_");
        <Gamma as toql::query_fields::QueryFields>::FieldsType::from_path(self.0)
    }
}

impl toql::sql_mapper::mapped::Mapped for Alpha {
    fn type_name() -> String {
        String::from("Alpha")
    }
    fn table_name() -> String {
        String::from("Alpha")
    }
    fn table_alias() -> String {
        String::from("alpha")
    }
    fn map(mapper: &mut toql::sql_mapper::SqlMapper) -> toql::result::Result<()> {
        mapper.map_column_with_options(
            "id",
            "id",
            toql::sql_mapper::field_options::FieldOptions::new()
                .key(true)
                .preselect(true),
        );
        mapper.map_column_with_options(
            "text",
            "text",
            toql::sql_mapper::field_options::FieldOptions::new().preselect(true),
        );
        mapper.map_join_with_options(
            "beta",
            "Beta",
            toql::sql_mapper::join_type::JoinType::Inner,
            {
                let mut t = toql::sql_expr::SqlExpr::literal(
                    <Beta as toql::sql_mapper::mapped::Mapped>::table_name(),
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
                        let default_self_column = format!("beta_{}", other_column);
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
            toql::sql_mapper::join_options::JoinOptions::new().preselect(true),
        );
        mapper.map_merge(
            "gamma",
            "Gamma",
            {
                toql::sql_expr::SqlExpr::from(vec![
                    toql::sql_expr::SqlExprToken::Literal("JOIN ".to_string()),
                    toql::sql_expr::SqlExprToken::Literal("Alpha".to_string()),
                    toql::sql_expr::SqlExprToken::Literal(" ".to_string()),
                    toql::sql_expr::SqlExprToken::SelfAlias,
                ])
            },
            {
                {
                    let mut tokens: Vec<toql::sql_expr::SqlExprToken> = Vec::new();
                    <AlphaKey as toql::key::Key>::columns()
                        .iter()
                        .zip(<AlphaKey as toql::key::Key>::default_inverse_columns())
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
        );
        Ok(())
    }
}
impl toql::sql_mapper::mapped::Mapped for &Alpha {
    fn type_name() -> String {
        <Alpha as toql::sql_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Alpha as toql::sql_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Alpha as toql::sql_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::sql_mapper::SqlMapper) -> toql::result::Result<()> {
        <Alpha as toql::sql_mapper::mapped::Mapped>::map(mapper)
    }
}
impl toql::sql_mapper::mapped::Mapped for &mut Alpha {
    fn type_name() -> String {
        <Alpha as toql::sql_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Alpha as toql::sql_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Alpha as toql::sql_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::sql_mapper::SqlMapper) -> toql::result::Result<()> {
        <Alpha as toql::sql_mapper::mapped::Mapped>::map(mapper)
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
                "beta" => {
                    return Ok(
                        <Join<Beta> as toql::tree::tree_insert::TreeInsert>::columns(
                            &mut descendents,
                        )?,
                    );
                }
                "gamma" => {
                    return Ok(<Gamma as toql::tree::tree_insert::TreeInsert>::columns(
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
                e.push_literal("id");
                e.push_literal(", ");
                e.push_literal("text");
                e.push_literal(", ");
                for other_column in
                    <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                {
                    let default_self_column = format!("beta_{}", other_column);
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
                "beta" => <Join<Beta> as toql::tree::tree_insert::TreeInsert>::values(
                    &self.beta,
                    &mut descendents,
                    roles,
                    values,
                )?,
                "gamma" => {
                    for f in &self.gamma {
                        <Gamma as toql::tree::tree_insert::TreeInsert>::values(
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
                &toql::key::Key::params(&<Join<Beta> as toql::keyed::Keyed>::key(&self.beta))
                    .into_iter()
                    .for_each(|a| {
                        values.push_arg(a);
                        values.push_literal(", ");
                    });
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
                "beta" => <Join<Beta> as toql::tree::tree_update::TreeUpdate>::update(
                    &self.beta,
                    &mut descendents,
                    fields,
                    roles,
                    exprs,
                )?,
                "gamma" => {
                    for f in &self.gamma {
                        <Gamma as toql::tree::tree_update::TreeUpdate>::update(
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
                if (path_selected || fields.contains("beta")) {
                    let inverse_columns =
                        <<Join<Beta> as toql::keyed::Keyed>::Key as toql::key::Key>::columns()
                            .iter()
                            .enumerate()
                            .map(|(i, c)| format!("beta_{}", c))
                            .collect::<Vec<String>>();
                    let args = toql::key::Key::params(&toql::keyed::Keyed::key(&self.beta));
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

impl<R, E> toql::backend::api::Load<R, E> for Beta
where
    Self: toql::keyed::Keyed
        + toql::sql_mapper::mapped::Mapped
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
impl<R, E> toql::backend::api::Load<R, E> for &Beta
where
    Self: toql::keyed::Keyed
        + toql::sql_mapper::mapped::Mapped
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
impl toql::backend::api::Insert for Beta where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::sql_mapper::mapped::Mapped
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::api::Insert for &mut Beta where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::sql_mapper::mapped::Mapped
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::api::Update for Beta where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::sql_mapper::mapped::Mapped
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::api::Update for &mut Beta where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::sql_mapper::mapped::Mapped
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::api::Count for Beta where
    Self: toql::keyed::Keyed + toql::sql_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::api::Count for &Beta where
    Self: toql::keyed::Keyed + toql::sql_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::api::Delete for Beta where
    Self: toql::sql_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
{
}
impl toql::backend::api::Delete for &Beta where
    Self: toql::sql_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
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
    fn auto_id() -> bool {
        false
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
    fn auto_id() -> bool {
        <Beta as toql::tree::tree_identity::TreeIdentity>::auto_id()
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
        registry: &mut toql::sql_mapper_registry::SqlMapperRegistry,
    ) -> toql::result::Result<()> {
        if registry.get("Beta").is_none() {
            registry.insert_new_mapper::<Beta>()?;
        }
        Ok(())
    }
}
impl toql::tree::tree_map::TreeMap for &Beta {
    fn map(
        registry: &mut toql::sql_mapper_registry::SqlMapperRegistry,
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

impl toql::sql_mapper::mapped::Mapped for Beta {
    fn type_name() -> String {
        String::from("Beta")
    }
    fn table_name() -> String {
        String::from("Beta")
    }
    fn table_alias() -> String {
        String::from("beta")
    }
    fn map(mapper: &mut toql::sql_mapper::SqlMapper) -> toql::result::Result<()> {
        mapper.map_column_with_options(
            "id",
            "id",
            toql::sql_mapper::field_options::FieldOptions::new()
                .key(true)
                .preselect(true),
        );
        mapper.map_column_with_options(
            "text",
            "text",
            toql::sql_mapper::field_options::FieldOptions::new().preselect(true),
        );
        Ok(())
    }
}
impl toql::sql_mapper::mapped::Mapped for &Beta {
    fn type_name() -> String {
        <Beta as toql::sql_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Beta as toql::sql_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Beta as toql::sql_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::sql_mapper::SqlMapper) -> toql::result::Result<()> {
        <Beta as toql::sql_mapper::mapped::Mapped>::map(mapper)
    }
}
impl toql::sql_mapper::mapped::Mapped for &mut Beta {
    fn type_name() -> String {
        <Beta as toql::sql_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Beta as toql::sql_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Beta as toql::sql_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::sql_mapper::SqlMapper) -> toql::result::Result<()> {
        <Beta as toql::sql_mapper::mapped::Mapped>::map(mapper)
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

impl<R, E> toql::backend::api::Load<R, E> for Gamma
where
    Self: toql::keyed::Keyed
        + toql::sql_mapper::mapped::Mapped
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
impl<R, E> toql::backend::api::Load<R, E> for &Gamma
where
    Self: toql::keyed::Keyed
        + toql::sql_mapper::mapped::Mapped
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
impl toql::backend::api::Insert for Gamma where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::sql_mapper::mapped::Mapped
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::api::Insert for &mut Gamma where
    Self: toql::tree::tree_insert::TreeInsert
        + toql::sql_mapper::mapped::Mapped
        + toql::tree::tree_identity::TreeIdentity
{
}
impl toql::backend::api::Update for Gamma where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::sql_mapper::mapped::Mapped
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::api::Update for &mut Gamma where
    Self: toql::tree::tree_update::TreeUpdate
        + toql::sql_mapper::mapped::Mapped
        + toql::tree::tree_identity::TreeIdentity
        + toql::tree::tree_predicate::TreePredicate
        + toql::tree::tree_insert::TreeInsert
{
}
impl toql::backend::api::Count for Gamma where
    Self: toql::keyed::Keyed + toql::sql_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::api::Count for &Gamma where
    Self: toql::keyed::Keyed + toql::sql_mapper::mapped::Mapped + std::fmt::Debug
{
}
impl toql::backend::api::Delete for Gamma where
    Self: toql::sql_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
{
}
impl toql::backend::api::Delete for &Gamma where
    Self: toql::sql_mapper::mapped::Mapped + toql::tree::tree_map::TreeMap + std::fmt::Debug
{
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct GammaKey {
    pub id: u64,
}
impl toql::key_fields::KeyFields for GammaKey {
    type Entity = Gamma;
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
impl toql::key_fields::KeyFields for &GammaKey {
    type Entity = Gamma;
    fn fields() -> Vec<String> {
        <GammaKey as toql::key_fields::KeyFields>::fields()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <GammaKey as toql::key_fields::KeyFields>::params(self)
    }
}
impl toql::key::Key for GammaKey {
    type Entity = Gamma;
    fn columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("id"));
        columns
    }
    fn default_inverse_columns() -> Vec<String> {
        let mut columns: Vec<String> = Vec::new();
        columns.push(String::from("gamma_id"));
        columns
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        let mut params: Vec<toql::sql_arg::SqlArg> = Vec::new();
        let key = self;
        params.push(toql::sql_arg::SqlArg::from(&key.id));
        params
    }
}
impl toql::key::Key for &GammaKey {
    type Entity = Gamma;
    fn columns() -> Vec<String> {
        <GammaKey as toql::key::Key>::columns()
    }
    fn default_inverse_columns() -> Vec<String> {
        <GammaKey as toql::key::Key>::default_inverse_columns()
    }
    fn params(&self) -> Vec<toql::sql_arg::SqlArg> {
        <GammaKey as toql::key::Key>::params(self)
    }
}
impl From<GammaKey> for toql::sql_arg::SqlArg {
    fn from(t: GammaKey) -> toql::sql_arg::SqlArg {
        toql::sql_arg::SqlArg::from(t.id)
    }
}
impl From<&GammaKey> for toql::sql_arg::SqlArg {
    fn from(t: &GammaKey) -> toql::sql_arg::SqlArg {
        toql::sql_arg::SqlArg::from(t.id.to_owned())
    }
}
impl toql::keyed::Keyed for Gamma {
    type Key = GammaKey;
    fn key(&self) -> Self::Key {
        GammaKey {
            id: self.id.to_owned(),
        }
    }
}
impl toql::keyed::Keyed for &Gamma {
    type Key = GammaKey;
    fn key(&self) -> Self::Key {
        <Gamma as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::Keyed for &mut Gamma {
    type Key = GammaKey;
    fn key(&self) -> Self::Key {
        <Gamma as toql::keyed::Keyed>::key(self)
    }
}
impl toql::keyed::KeyedMut for Gamma {
    fn set_key(&mut self, key: Self::Key) {
        self.id = key.id;
    }
}
impl toql::keyed::KeyedMut for &mut Gamma {
    fn set_key(&mut self, key: Self::Key) {
        <Gamma as toql::keyed::KeyedMut>::set_key(self, key)
    }
}
impl std::convert::TryFrom<Vec<toql::sql_arg::SqlArg>> for GammaKey {
    type Error = toql::error::ToqlError;
    fn try_from(args: Vec<toql::sql_arg::SqlArg>) -> toql::result::Result<Self> {
        use std::convert::TryInto;
        Ok(GammaKey {
            id: args
                .get(0)
                .ok_or(toql::error::ToqlError::ValueMissing("id".to_string()))?
                .try_into()?,
        })
    }
}
impl std::convert::From<u64> for GammaKey {
    fn from(key: u64) -> Self {
        Self { id: key }
    }
}
impl std::hash::Hash for Gamma {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        <Gamma as toql::keyed::Keyed>::key(self).hash(state);
    }
}

impl toql::tree::tree_identity::TreeIdentity for Gamma {
    fn auto_id() -> bool {
        false
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
impl toql::tree::tree_identity::TreeIdentity for &mut Gamma {
    fn auto_id() -> bool {
        <Gamma as toql::tree::tree_identity::TreeIdentity>::auto_id()
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
        <Gamma as toql::tree::tree_identity::TreeIdentity>::set_id(self, descendents, action)
    }
}
impl toql::tree::tree_map::TreeMap for Gamma {
    fn map(
        registry: &mut toql::sql_mapper_registry::SqlMapperRegistry,
    ) -> toql::result::Result<()> {
        if registry.get("Gamma").is_none() {
            registry.insert_new_mapper::<Gamma>()?;
        }
        Ok(())
    }
}
impl toql::tree::tree_map::TreeMap for &Gamma {
    fn map(
        registry: &mut toql::sql_mapper_registry::SqlMapperRegistry,
    ) -> toql::result::Result<()> {
        <Gamma as toql::tree::tree_map::TreeMap>::map(registry)
    }
}
impl toql::tree::tree_predicate::TreePredicate for Gamma {
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
impl toql::tree::tree_predicate::TreePredicate for &Gamma {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Gamma as toql::tree::tree_predicate::TreePredicate>::columns(self, descendents)
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
        <Gamma as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl toql::tree::tree_predicate::TreePredicate for &mut Gamma {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        &self,
        mut descendents: &mut I,
    ) -> std::result::Result<Vec<String>, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Gamma as toql::tree::tree_predicate::TreePredicate>::columns(self, descendents)
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
        <Gamma as toql::tree::tree_predicate::TreePredicate>::args(self, descendents, args)
    }
}
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for Gamma
where
    E: std::convert::From<toql::error::ToqlError>,
    GammaKey: toql::from_row::FromRow<R, E>,
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
                    let fk = GammaKey::from_row(&row, &mut i, &mut iter)?.ok_or(
                        toql::error::ToqlError::ValueMissing(
                            <GammaKey as toql::key::Key>::columns().join(", "),
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
impl<R, E> toql::tree::tree_index::TreeIndex<R, E> for &Gamma
where
    E: std::convert::From<toql::error::ToqlError>,
    GammaKey: toql::from_row::FromRow<R, E>,
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
        <Gamma as toql::tree::tree_index::TreeIndex<R, E>>::index(
            descendents,
            rows,
            row_offset,
            index,
        )
    }
}
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for Gamma
where
    E: std::convert::From<toql::error::ToqlError>,
    GammaKey: toql::from_row::FromRow<R, E>,
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
                let pk: GammaKey = <Self as toql::keyed::Keyed>::key(&self);
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
impl<R, E> toql::tree::tree_merge::TreeMerge<R, E> for &mut Gamma
where
    E: std::convert::From<toql::error::ToqlError>,
    GammaKey: toql::from_row::FromRow<R, E>,
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
        <Gamma as toql::tree::tree_merge::TreeMerge<R, E>>::merge(
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

impl<R, E> toql::from_row::FromRow<R, E> for GammaKey
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
    ) -> std::result::Result<Option<GammaKey>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        Ok(Some(GammaKey {
            id: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "GammaKey::id".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl<R, E> toql::from_row::FromRow<R, E> for Gamma
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
    #[allow(unused_variables, unused_mut)]
    fn from_row<'a, I>(
        mut row: &R,
        i: &mut usize,
        mut iter: &mut I,
    ) -> std::result::Result<Option<Gamma>, E>
    where
        I: Iterator<Item = &'a toql::sql_builder::select_stream::Select> + Clone,
    {
        use toql::sql_builder::select_stream::Select;
        Ok(Some(Gamma {
            id: {
                match <u64 as toql::from_row::FromRow<_, E>>::from_row(row, i, iter)? {
                    Some(s) => s,
                    _ => return Ok(None),
                }
            },
            text: {
                toql::from_row::FromRow::<_, E>::from_row(row, i, iter)?.ok_or(
                    toql::deserialize::error::DeserializeError::SelectionExpected(
                        "Gamma::text".to_string(),
                    )
                    .into(),
                )?
            },
        }))
    }
}

impl toql::query_fields::QueryFields for Gamma {
    type FieldsType = GammaFields;
    fn fields() -> GammaFields {
        GammaFields::new()
    }
    fn fields_from_path(path: String) -> GammaFields {
        GammaFields::from_path(path)
    }
}
pub struct GammaFields(String);
impl toql::query_path::QueryPath for GammaFields {
    fn into_path(self) -> String {
        self.0
    }
}
impl GammaFields {
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

impl toql::sql_mapper::mapped::Mapped for Gamma {
    fn type_name() -> String {
        String::from("Gamma")
    }
    fn table_name() -> String {
        String::from("Gamma")
    }
    fn table_alias() -> String {
        String::from("gamma")
    }
    fn map(mapper: &mut toql::sql_mapper::SqlMapper) -> toql::result::Result<()> {
        mapper.map_column_with_options(
            "id",
            "id",
            toql::sql_mapper::field_options::FieldOptions::new()
                .key(true)
                .preselect(true),
        );
        mapper.map_column_with_options(
            "text",
            "text",
            toql::sql_mapper::field_options::FieldOptions::new().preselect(true),
        );
        Ok(())
    }
}
impl toql::sql_mapper::mapped::Mapped for &Gamma {
    fn type_name() -> String {
        <Gamma as toql::sql_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Gamma as toql::sql_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Gamma as toql::sql_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::sql_mapper::SqlMapper) -> toql::result::Result<()> {
        <Gamma as toql::sql_mapper::mapped::Mapped>::map(mapper)
    }
}
impl toql::sql_mapper::mapped::Mapped for &mut Gamma {
    fn type_name() -> String {
        <Gamma as toql::sql_mapper::mapped::Mapped>::type_name()
    }
    fn table_name() -> String {
        <Gamma as toql::sql_mapper::mapped::Mapped>::table_name()
    }
    fn table_alias() -> String {
        <Gamma as toql::sql_mapper::mapped::Mapped>::table_alias()
    }
    fn map(mapper: &mut toql::sql_mapper::SqlMapper) -> toql::result::Result<()> {
        <Gamma as toql::sql_mapper::mapped::Mapped>::map(mapper)
    }
}

impl toql::tree::tree_insert::TreeInsert for Gamma {
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
impl toql::tree::tree_insert::TreeInsert for &Gamma {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Gamma as toql::tree::tree_insert::TreeInsert>::columns(descendents)
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
        <Gamma as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, values)
    }
}
impl toql::tree::tree_insert::TreeInsert for &mut Gamma {
    #[allow(unused_mut)]
    fn columns<'a, I>(
        mut descendents: &mut I,
    ) -> std::result::Result<toql::sql_expr::SqlExpr, toql::error::ToqlError>
    where
        I: Iterator<Item = toql::query::field_path::FieldPath<'a>>,
    {
        <Gamma as toql::tree::tree_insert::TreeInsert>::columns(descendents)
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
        <Gamma as toql::tree::tree_insert::TreeInsert>::values(self, descendents, roles, values)
    }
}

impl toql::tree::tree_update::TreeUpdate for Gamma {
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
                expr.push_literal("Gamma");
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
                        toql::sql_expr::resolver::Resolver::new().with_self_alias("Gamma");
                    expr.extend(resolver.alias_to_literals(&toql::key::Key::predicate_expr(&key))?);
                    exprs.push(expr);
                }
            }
        };
        Ok(())
    }
}
impl toql::tree::tree_update::TreeUpdate for &mut Gamma {
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
        <Gamma as toql::tree::tree_update::TreeUpdate>::update(
            self,
            descendents,
            fields,
            roles,
            exprs,
        )
    }
}
