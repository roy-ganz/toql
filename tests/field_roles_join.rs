use std::collections::HashSet;

use pretty_assertions::assert_eq;
use toql::mock_db::MockDb;
use toql::prelude::{
    fields, query, Cache, ContextBuilder, Join, SqlBuilderError, Toql, ToqlApi, ToqlError,
};
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
#[toql(auto_key)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    #[toql(roles(load = "load1_role", update = "update1_text_role"))]
    text: Option<String>,

    #[toql(join, roles(load = "load2_role", update = "update1_level2_role"))]
    level2: Option<Join<Level2>>,
}
#[derive(Debug, Default, Toql)]
#[toql(auto_key)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    #[toql(roles(load = "load2_text_role", update = "update2_text_role"))]
    text: Option<String>, // Selectable

    #[toql(join, roles(load = "load2_level3_role"))]
    level3: Join<Level3>,
}

#[derive(Debug, Default, Toql)]
#[toql(auto_key)]
pub struct Level3 {
    #[toql(key)]
    id: u64,
    #[toql(roles(load = "load3_text_role", update = "update3_text_role"))]
    text: String, // Preselected
}

fn populated_level() -> Level1 {
    let l3 = Level3 {
        id: 3,
        text: "level3".to_string(),
    };
    let l2 = Level2 {
        id: 2,
        text: Some("level2".to_string()),
        level3: Join::with_entity(l3),
    };

    Level1 {
        id: 1,
        text: Some("level1".to_string()),
        level2: Some(Join::with_entity(l2)),
    }
}

fn toql_for_roles<'a>(role_names: &[&str], cache: &'a Cache) -> MockDb<'a> {
    let mut roles = HashSet::new();
    role_names.into_iter().for_each(|r| {
        roles.insert(r.to_string());
    });

    let context = ContextBuilder::new().with_roles(roles).build();
    MockDb::with_context(cache, context)
}

#[tokio::test]
#[traced_test("info")]
async fn load() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load level 1
    // No role, only id is loaded,
    // Selectable join and merge do not required roles
    let q = query!(Level1, "*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id FROM Level1 level1"
    );

    // Load level 1
    // fail to load text
    let q = query!(Level1, "text");
    let err = toql.load_many(&q).await.err().unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::RoleRequired(
            "load1_role".to_string(),
            "field `text`".to_string()
        ))
        .to_string()
    );

    // Load level 1
    // load1_role: only id, text is loaded
    let mut toql = toql_for_roles(&["load1_role"], &cache);
    let q = query!(Level1, "*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text FROM Level1 level1"
    );

    // fail to load join
    let q = query!(Level1, "level2_*");
    let err = toql.load_many(q).await.err().unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::RoleRequired(
            "load2_role".to_string(),
            "path `level2`".to_string()
        ))
        .to_string()
    );

    // Load level 1, 2
    // Fails, because preselected join is missing role
    let mut toql = toql_for_roles(&["load2_role"], &cache);
    let q = query!(Level1, "level2_*");
    let err = toql.load_many(q).await.err().unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::RoleRequired(
            "load2_level3_role".to_string(),
            "path `level2_level3`".to_string()
        ))
        .to_string()
    );

    // Load level 1, 2 : level1_id, level2_id, level2_text,
    //  level2_level3_id (preselected), level2_level3_text (preselected)
    // failes, because role is missing for level3
    let mut toql = toql_for_roles(&["load2_role", "load2_level3_role"], &cache);
    let q = query!(Level1, "level2_*");
    let err = toql.load_many(q).await.err().unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::RoleRequired(
            "load3_text_role".to_string(),
            "field `level2_level3_text`".to_string()
        ))
        .to_string()
    );

    // Load level 1, 2 : level1_id, level2_id, level2_text,
    //  level2_level3_id (preselected) , level2_level3_text (preselected)
    let mut toql = toql_for_roles(
        &[
            "load2_role",
            "load2_text_role",
            "load2_level3_role",
            "load3_text_role",
        ],
        &cache,
    );
    let q = query!(Level1, "level2_*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id, level1_level2.text, level1_level2_level3.id, level1_level2_level3.text \
            FROM Level1 level1 \
            JOIN (Level2 level1_level2 \
                JOIN (Level3 level1_level2_level3) ON (level1_level2.level3_id = level1_level2_level3.id)) \
            ON (level1.level2_id = level1_level2.id)"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn load2() {
    let cache = Cache::new();

    let mut toql = MockDb::from(&cache);
    let q = query!(Level1, "+text");
    let err = toql.load_many(&q).await.err().unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::RoleRequired(
            "load1_role".to_string(),
            "field `text`".to_string()
        ))
        .to_string()
    );

    let mut toql = toql_for_roles(&["load1_role"], &cache);
    let q = query!(Level1, "+text");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text FROM Level1 level1 ORDER BY level1.text ASC"
    );
    let q = query!(Level1, "+.text");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id FROM Level1 level1 ORDER BY level1.text ASC"
    );
    let q = query!(Level1, "+.text eq 'ABC'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id FROM Level1 level1 WHERE level1.text = 'ABC' ORDER BY level1.text ASC"
    );

    // Load preselects with hidden filter on level2.text
    let mut toql = toql_for_roles(
        &[
            "load1_role",
            "load2_role",
            "load2_text_role",
            "load2_level3_role",
            "load3_text_role",
        ],
        &cache,
    );
    let q = query!(Level1, "+.level2_text eq 'ABC'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id, level1_level2_level3.id, level1_level2_level3.text \
            FROM Level1 level1 \
            JOIN (Level2 level1_level2 \
                JOIN (Level3 level1_level2_level3) \
                ON (level1_level2.level3_id = level1_level2_level3.id)) \
            ON (level1.level2_id = level1_level2.id) \
            WHERE level1_level2.text = \'ABC\' \
            ORDER BY level1_level2.text ASC"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn update() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    let mut l = populated_level();

    // Update level 1: No Role
    // - role restricted fields are skipped
    // - no field is updated
    // - no SQL is generated
    assert!(toql.update_one(&mut l, fields!(Level1, "*")).await.is_ok());
    assert!(toql.sqls_empty());

    // Update level 1 text: No Role
    // - Fail to update level1.text because role is missing
    let err = toql
        .update_one(&mut l, fields!(Level1, "text"))
        .await
        .err()
        .unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::RoleRequired(
            "update1_text_role".to_string(),
            "field `text` on mapper `Level1`".to_string()
        ))
        .to_string()
    );

    // Update level 1 text with role
    let mut toql = toql_for_roles(&["update1_text_role"], &cache);
    assert!(toql.update_one(&mut l, fields!(Level1, "*")).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level1 SET text = 'level1' WHERE id = 1"
    );

    let mut toql = toql_for_roles(&["update1_text_role", "update1_level2_role"], &cache);
    assert!(toql.update_one(&mut l, fields!(Level1, "*")).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level1 SET text = 'level1', level2_id = 2 WHERE id = 1"
    );

    // Update level3 text (preselected field) without roles.
    // No SQL is generated, field is skipped because role `update2_text_role` is missing
    let mut toql = toql_for_roles(&[], &cache);
    assert!(toql
        .update_one(&mut l, fields!(Level1, "level2_level3_*"))
        .await
        .is_ok());
    assert!(toql.sqls_empty());
}
