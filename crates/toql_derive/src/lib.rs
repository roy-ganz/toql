//! The `#[derive(Toql)]` creates all the boilerplate code to make the ✨ happen.
//! Using the derive is the easy. However beware that the generated code size can become large
//! as it's about ~3K lines of code for a medium `struct`.
//!
//! For a bigger project, you are strongly advised to create a cargo workspace and
//! to put your Toql derived structs into a separate crate to reduce compile time.
//! This will pay off once your database model stabilizes.
//!
//! The `#[derive(ToqlEnum)]` must be added on enums to implement deserialization and conversion.
//! Notice that `ToqlEnum` requires enums to have implementations for the `ToString` and `FromStr` traits.

#![recursion_limit = "1024"]

extern crate proc_macro;

extern crate syn;

#[macro_use]
extern crate quote;

use parsed::parsed_struct::ParsedStruct;
use proc_macro::TokenStream;
use quote::ToTokens;

mod attr;
mod error;
mod parsed;
mod result;
mod to_tokens;

/// Derive to add Toql functionality to your struct.
#[proc_macro_derive(Toql, attributes(toql))]
pub fn toql_derive(input: TokenStream) -> TokenStream {
    let _ = env_logger::try_init(); // Avoid multiple init
    let parsed = syn::parse::<ParsedStruct>(input);
    TokenStream::from(match parsed {
        Ok(gen) => gen.into_token_stream(),
        Err(error) => error.to_compile_error(),
    })
}

#[test]
fn test_predicate() {
    let input = r#"
    #[toql(auto_key, predicate(
        name = "search",
        sql = "MATCH (..id, ..currency_symbol) AGAINST (?  IN BOOLEAN MODE)",
        on_aux_param(name= "hello", index= 0), on_aux_param( name = "ajl", index = 3),
        handler="test::get_handler"
    ))]
    struct User {
        #[toql(key)]
        id: u64,
        name: String
    }"#;

    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    let pred_arg = parsed.predicates.get("search").unwrap();
    assert_eq!(
        pred_arg.sql,
        "MATCH (..id, ..currency_symbol) AGAINST (?  IN BOOLEAN MODE)"
    );
    assert!(!parsed.to_token_stream().is_empty());

    let input = r#"
    #[toql(predicate(
        name = "long_name",
        sql = "true",
    ))]
    struct User {
        #[toql(key)]
        id: u64,
        other: Other
    }"#;
    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    let pred_arg = parsed.predicates.get("longName").unwrap();
    assert_eq!(pred_arg.sql, "true");
    assert!(!parsed.to_token_stream().is_empty())
}
#[test]
fn test_selection() {
    let input = r#"
    #[toql(selection(
        name = "std",
        fields = "*, other_id",
    ))]
    struct User {
        #[toql(key)]
        id: u64,
        other: Other
    }"#;
    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    let sel_arg = parsed.selections.get("std").unwrap();
    assert_eq!(sel_arg.fields, "*, other_id");
    assert!(!parsed.to_token_stream().is_empty());

    let input = r#"
    #[toql(selection(
        name = "cnt",
        fields = "*, other_id",
    ))]
    struct User {
        #[toql(key)]
        id: u64,
        other: Other
    }"#;
    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());
    assert!(!parsed.unwrap().to_token_stream().is_empty());

    let input = r#"
    #[toql(selection(
        name = "long_name",
        fields = "*",
    ))]
    struct User {
        #[toql(key)]
        id: u64,
        other: Other
    }"#;
    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    let sel_arg = parsed.selections.get("longName").unwrap();
    assert_eq!(sel_arg.fields, "*");
    assert!(!parsed.to_token_stream().is_empty())
}
#[test]
fn test_invalid_selection() {
    let input = r#"
    #[toql(selection(
        name = "st",
        fields = "*, other_id",
    ))]
    struct User {
        #[toql(key)]
        id: u64,
        other: Other
    }"#;
    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_err());

    let input = r#"
    #[toql(selection(
        name = "",
        fields = "*, other_id",
    ))]
    struct User {
        #[toql(key)]
        id: u64,
        other: Other
    }"#;
    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_err());
}
#[test]
fn test_valid_key() {
    use crate::parsed::field::field_kind::FieldKind;
    let input = r#"
        struct User {
            #[toql(key)]
            id: u64,
            name: String
        }"#;
    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    let field = parsed.fields.get(0).unwrap();
    assert!(matches!(&field.kind, FieldKind::Regular(regular_kind) if regular_kind.key));
    assert!(!parsed.to_token_stream().is_empty());

    let input = r#"
        struct User {
            #[toql(key)]
            id1: u64,
            #[toql(key)]
            id2: String,
            name: String
        }"#;
    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    let field = parsed.fields.get(0).unwrap();
    assert!(matches!(&field.kind, FieldKind::Regular(regular_kind) if regular_kind.key));
    let field = parsed.fields.get(1).unwrap();
    assert!(matches!(&field.kind, FieldKind::Regular(regular_kind) if regular_kind.key));
    assert!(!parsed.to_token_stream().is_empty())
}
#[test]
fn test_auto_key() {
    let input = r#"
        #[toql(auto_key)]
        struct User {
            #[toql(key)]
            id: u64,
            name: String
        }"#;
    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    assert!(parsed.auto_key);
    assert!(!parsed.to_token_stream().is_empty());
}
#[test]
fn test_invalid_key() {
    let optional_key_input = r#"
        struct User {
            #[toql(key)]
            id: Option<u64>,
            name: String
        }"#;
    let parsed = syn::parse_str::<ParsedStruct>(optional_key_input);
    assert!(parsed.is_err());

    let missing_key_input = r#"
        struct User {
            id: u64,
            name: String
        }"#;
    let parsed = syn::parse_str::<ParsedStruct>(missing_key_input);
    assert!(parsed.is_err());

    let trailing_key = r#"
        struct User {
            id: u64,
            #[toql(key)]
            name: String
        }"#;
    let parsed = syn::parse_str::<ParsedStruct>(trailing_key);
    assert!(parsed.is_err());
}

#[test]
fn test_field_types() {
    use crate::parsed::field::{
        field_kind::FieldKind,
        regular_field::{RegularSelection, SqlTarget},
    };

    let input = r#"
    #[toql(auto_key)]
    struct User {
        #[toql(key)]
        id: u64,
        name1: String,
        
        #[toql(sql="(SELECT 'ABC')")]
        name2: Option<String>,

        #[toql(preselect, column="name2_column", handler="test::field_handler")]
        name3: Option<String>,
        name4: Option<Option<std::string::String>>
    }"#;

    let parsed = syn::parse_str::<ParsedStruct>(input);
    println!("{:?}", &parsed);
    assert!(parsed.is_ok());

    let parsed = parsed.unwrap();

    let field = parsed.fields.get(0).unwrap();
    assert!(matches!(&field.kind, FieldKind::Regular(regular_kind)
            if regular_kind.selection == RegularSelection::Preselect));

    let field = parsed.fields.get(1).unwrap();
    assert!(matches!(&field.kind, FieldKind::Regular(regular_kind)
            if regular_kind.selection == RegularSelection::Preselect));

    let field = parsed.fields.get(2).unwrap();
    let regular_kind = field.kind.as_regular().unwrap();
    assert_eq!(regular_kind.selection, RegularSelection::Select);
    assert_eq!(
        regular_kind.sql_target,
        SqlTarget::Expression("(SELECT 'ABC')".to_string())
    );

    let field = parsed.fields.get(3).unwrap();
    let regular_kind = field.kind.as_regular().unwrap();
    assert_eq!(regular_kind.selection, RegularSelection::PreselectNullable);
    assert_eq!(
        regular_kind.handler.as_ref().unwrap(),
        &syn::parse_str::<syn::Path>("test::field_handler").unwrap()
    );
    assert_eq!(
        regular_kind.sql_target,
        SqlTarget::Column("name2_column".to_string())
    );

    let field = parsed.fields.get(4).unwrap();
    assert!(matches!(&field.kind, FieldKind::Regular(regular_kind)
            if regular_kind.selection == RegularSelection::SelectNullable));

    assert!(!parsed.to_token_stream().is_empty())
}

#[test]
fn test_join_types() {
    use crate::parsed::field::field_kind::FieldKind;
    use crate::parsed::field::join_field::JoinSelection;

    let input = r#"
    #[toql(auto_key)]
    struct User {
        #[toql(key, join)]
        join1: Other,

        #[toql(join())]
        join2: Other,

        #[toql(preselect, join)]
        join3: Option<Other>,

        #[toql(join)]
        join4: Option<Other>,

        #[toql(join)]
        join5: Option<Option<Other>>,

        #[toql(join)]
        join6: Join<Other>,

        #[toql(preselect, join)]
        join7: Option<Join<Other>>,

        #[toql(join)]
        join8: Option<Join<Other>>,

        #[toql(join)]
        join9: Option<Option<Join<Other>>>,
    }"#;

    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());

    let parsed = parsed.unwrap();
    let field = parsed.fields.get(0).unwrap();
    assert!(matches!(&field.kind, FieldKind::Join(join_kind)
            if join_kind.selection == JoinSelection::PreselectInner));

    let field = parsed.fields.get(1).unwrap();
    assert!(matches!(&field.kind, FieldKind::Join(join_kind)
            if join_kind.selection == JoinSelection::PreselectInner));

    let field = parsed.fields.get(2).unwrap();
    assert!(matches!(&field.kind, FieldKind::Join(join_kind)
            if join_kind.selection == JoinSelection::PreselectLeft));

    let field = parsed.fields.get(3).unwrap();
    assert!(matches!(&field.kind, FieldKind::Join(join_kind)
            if join_kind.selection == JoinSelection::SelectInner));

    let field = parsed.fields.get(4).unwrap();
    assert!(matches!(&field.kind, FieldKind::Join(join_kind)
            if join_kind.selection == JoinSelection::SelectLeft));

    let field = parsed.fields.get(5).unwrap();
    assert!(matches!(&field.kind, FieldKind::Join(join_kind)
            if join_kind.selection == JoinSelection::PreselectInner));

    let field = parsed.fields.get(6).unwrap();
    assert!(matches!(&field.kind, FieldKind::Join(join_kind)
            if join_kind.selection == JoinSelection::PreselectLeft));

    let field = parsed.fields.get(7).unwrap();
    assert!(matches!(&field.kind, FieldKind::Join(join_kind)
            if join_kind.selection == JoinSelection::SelectInner));

    let field = parsed.fields.get(8).unwrap();
    assert!(matches!(&field.kind, FieldKind::Join(join_kind)
            if join_kind.selection == JoinSelection::SelectLeft));

    assert!(!parsed.to_token_stream().is_empty())
}
#[test]
fn test_partial_joins() {
    use crate::parsed::field::field_kind::FieldKind;

    let input = r#"
    #[toql(auto_key)]
    struct User {
        #[toql(key)]
        id: u64,
        #[toql(join(partial_table))]
        join1 : Join<Other>,
    }"#;

    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());

    let parsed = parsed.unwrap();
    let field = parsed.fields.get(1).unwrap();
    assert!(matches!(&field.kind, FieldKind::Join(join_kind) if join_kind.partial_table));

    assert!(!parsed.to_token_stream().is_empty())
}
#[test]
fn test_merge_types() {
    use crate::parsed::field::field_kind::FieldKind;
    use crate::parsed::field::merge_field::MergeColumn;
    use crate::parsed::field::merge_field::MergeSelection;

    let input = r#"
    #[toql(auto_key)]
    struct User {
        #[toql(key)]
        id: u64,
        #[toql(merge)]
        merge1 : Vec<Other>,
        #[toql(merge)]
        merge2 : Option<Vec<Other>>,
        #[toql(merge(columns(self = "id", other = "level3_id"), join_sql = "...text = 'ABC'"))]
        merge3 : Vec<Other>,
        #[toql(merge(columns(self = "id", other = "other.id"),columns(self = "id2", other = "other.id2")))]
        merge4 : Vec<Other>
    }"#;

    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());

    let parsed = parsed.unwrap();
    let field = parsed.fields.get(1).unwrap();
    assert!(matches!(&field.kind, FieldKind::Merge(merge_kind)
            if merge_kind.selection == MergeSelection::Preselect));

    let field = parsed.fields.get(2).unwrap();
    assert!(matches!(&field.kind, FieldKind::Merge(merge_kind) 
            if merge_kind.selection == MergeSelection::Select));

    let field = parsed.fields.get(3).unwrap();
    let merge_kind = field.kind.as_merge().unwrap();
    assert_eq!(merge_kind.selection, MergeSelection::Preselect);
    assert_eq!(merge_kind.join_sql.as_ref().unwrap(), "...text = 'ABC'");
    let col = merge_kind.columns.get(0).unwrap();
    assert_eq!(col.this, "id");
    matches!( &col.other, MergeColumn::Unaliased(u) if u == "level3_id");

    let field = parsed.fields.get(4).unwrap();
    let merge_kind = field.kind.as_merge().unwrap();
    assert_eq!(merge_kind.selection, MergeSelection::Preselect);
    let col = merge_kind.columns.get(0).unwrap();
    assert_eq!(col.this, "id");
    matches!( &col.other, MergeColumn::Aliased(u) if u == "id");
    let col = merge_kind.columns.get(1).unwrap();
    assert_eq!(col.this, "id2");
    matches!( &col.other, MergeColumn::Aliased(u) if u == "id2");

    assert!(!parsed.to_token_stream().is_empty())
}
#[test]
fn test_roles() {
    let input = r#"
    #[toql(roles(insert="role3", delete="role3,role4"))]
    struct User {
        #[toql(key)]
        id: u64,
        #[toql(roles(load="role1", update="role1,role2"))]
        field1 : Option<String>,
    }"#;

    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());

    let parsed = parsed.unwrap();
    assert_eq!(parsed.roles.insert, Some("role3".to_string()));
    assert_eq!(parsed.roles.delete, Some("role3,role4".to_string()));

    let field = parsed.fields.get(1).unwrap();
    assert_eq!(field.roles.load, Some("role1".to_string()));
    assert_eq!(field.roles.update, Some("role1,role2".to_string()));

    assert!(!parsed.to_token_stream().is_empty())
}

#[test]
fn test_skip() {
    use crate::parsed::field::field_kind::FieldKind;

    let input = r#"
    struct User {
        #[toql(key)]
        id: u64,
        #[toql(skip)]
        field1 : Option<String>,
        #[toql(skip_mut, skip_wildcard)]
        field2 : Option<String>,
    }"#;

    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    let field = parsed.fields.get(1).unwrap();
    assert!(&field.skip);
    let field = parsed.fields.get(2).unwrap();
    assert!(&field.skip_mut);
    assert!(matches!(&field.kind, FieldKind::Regular(regular_kind)
        if regular_kind.skip_wildcard));
    assert!(!parsed.to_token_stream().is_empty())
}
#[test]
fn test_invalid_skip() {
    let input = r#"
    struct User {
        #[toql(key)]
        id: u64,
        #[toql(skip, skip_mut, skip_wildcard)]
        field1 : Option<String>,
    }"#;

    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_err());

    let input = r#"
    struct User {
        #[toql(key)]
        id: u64,
        #[toql(join, skip_wildcard)]
        field1 : Option<String>,
    }"#;

    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_err());

    let input = r#"
    struct User {
        #[toql(key)]
        id: u64,
        #[toql(merge, skip_wildcard)]
        field1 : Vec<Other>,
    }"#;

    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_err());
}

#[test]
fn test_handler() {
    let input = r#"
        #[toql(handler="get_field_handler")]
        struct User {
            #[toql(key)]
            id: u64,
            name: String
        }"#;
    let parsed = syn::parse_str::<ParsedStruct>(input);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    assert!(parsed.field_handler.is_some());
}
