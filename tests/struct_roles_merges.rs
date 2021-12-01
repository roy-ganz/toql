use std::collections::HashSet;

use pretty_assertions::assert_eq;
use toql::mock_db::MockDb;
use toql::prelude::{
    fields, paths, query, Cache, ContextBuilder, SqlBuilderError, Toql, ToqlApi, ToqlError,
};
use toql::row;
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
#[toql(
    auto_key,
    roles(
        load = "load1_role",
        insert = "ins1_role",
        update = "upd1_role",
        delete = "del1_role"
    )
)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    text: Option<String>,

    #[toql(merge)]
    level2: Option<Vec<Level2>>,
}
#[derive(Debug, Default, Toql)]
#[toql(
    auto_key,
    roles(
        load = "load2_role",
        insert = "ins2_role",
        update = "upd2_role",
        delete = "del2_role"
    )
)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    #[toql(foreign_key)]
    level1_id: u64,
    text: Option<String>, // Selectable
}

fn populated_level() -> Level1 {
    let l2 = Level2 {
        id: 0, // New item
        level1_id: 1,
        text: Some("level2".to_string()),
    };

    Level1 {
        id: 1,
        text: Some("level1".to_string()),
        level2: Some(vec![l2]),
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
    // fail to load fields
    let q = query!(Level1, "*");
    let err = toql.load_many(q).await.err().unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::RoleRequired(
            "load1_role".to_string(),
            "mapper `Level1`".to_string()
        ))
        .to_string()
    );

    // Load level 1 with load1_role
    // succeed to load fields
    let mut toql = toql_for_roles(&["load1_role"], &cache);
    let q = query!(Level1, "*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text FROM Level1 level1"
    );

    let mut toql = toql_for_roles(&["load1_role"], &cache);
    toql.mock_rows("SELECT level1.id FROM Level1 level1", vec![row!(1u64)]);
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

    let mut toql = toql_for_roles(&["load1_role", "load2_role"], &cache);
    toql.mock_rows("SELECT level1.id FROM Level1 level1", vec![row!(1u64)]);
    toql.mock_rows(
        "SELECT level1.id, level2.id, level2.level1_id, level2.text \
                        FROM Level2 level2 \
                        JOIN Level1 level1 ON (level1.id = level2.level1_id AND level1.id = 1)",
        vec![row!(1u64, 2u64, 1u64, "level2")],
    );
    let q = query!(Level1, "level2_*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sqls(),
        [
            "SELECT level1.id FROM Level1 level1",
            "SELECT level1.id, level2.id, level2.level1_id, level2.text \
            FROM Level2 level2 \
            JOIN Level1 level1 ON (level1.id = level2.level1_id AND level1.id = 1)"
        ]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn update() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    let mut l = populated_level();

    // Update level 1: No Role
    // Throws errror
    let err = toql
        .update_one(&mut l, fields!(Level1, "*"))
        .await
        .err()
        .unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::RoleRequired(
            "upd1_role".to_string(),
            "mapper `Level1`".to_string()
        ))
        .to_string()
    );

    // Update level 1 text with role
    let mut toql = toql_for_roles(&["upd1_role"], &cache);
    assert!(toql.update_one(&mut l, fields!(Level1, "*")).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level1 SET text = 'level1' WHERE id = 1"
    );

    // Update level 2: Role for level2, role for leve1 is missing
    // That's fine, update fields from level2
    // No statements, because new vec item
    let mut toql = toql_for_roles(&["upd2_role"], &cache);
    assert!(toql
        .update_one(&mut l, fields!(Level1, "level2_*"))
        .await
        .is_ok());
    assert!(toql.sqls_empty());
}

#[tokio::test]
#[traced_test("info")]
async fn insert() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    let mut l = populated_level();

    // Insert level 1: No Role
    // Throws errror
    let err = toql
        .insert_one(&mut l, paths!(Level1, ""))
        .await
        .err()
        .unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::RoleRequired(
            "ins1_role".to_string(),
            "mapper `Level1`".to_string()
        ))
        .to_string()
    );

    // Insert level 1 with role
    let mut toql = toql_for_roles(&["ins1_role"], &cache);
    assert!(toql.insert_one(&mut l, paths!(Level1, "")).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "INSERT INTO Level1 (text) VALUES ('level1')"
    );

    // Insert level 2 with missing role for level1
    let mut toql = toql_for_roles(&["ins2_role"], &cache);
    let err = toql
        .insert_one(&mut l, paths!(Level1, "level2"))
        .await
        .err()
        .unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::RoleRequired(
            "ins1_role".to_string(),
            "mapper `Level1`".to_string()
        ))
        .to_string()
    );

    // Insert level 2 with roles
    // generated id (100) is updated on level1 and updated on Level2
    let mut toql = toql_for_roles(&["ins1_role", "ins2_role"], &cache);
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2"))
        .await
        .is_ok());

    assert_eq!(
        toql.take_unsafe_sqls(),
        [
            "INSERT INTO Level1 (text) VALUES (\'level1\')",
            "INSERT INTO Level2 (level1_id, text) VALUES (100, \'level2\')"
        ]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn delete() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Delete Level1 without role
    // Fails
    let q = query!(Level1, "id eq 1");
    let err = toql.delete_many(q).await.err().unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::RoleRequired(
            "del1_role".to_string(),
            "mapper `Level1`".to_string()
        ))
        .to_string()
    );

    // Delete Level1 with role
    let mut toql = toql_for_roles(&["del1_role"], &cache);
    let q = query!(Level1, "id eq 1");
    assert!(toql.delete_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "DELETE level1 FROM Level1 level1 WHERE level1.id = 1"
    );

    // Delete level 1 with level 2 merge and role for level1
    // Succeeds with no SQL, level2 is merge, so filter is skipped
    // Since there is no filter at all, delete refuses to be called (for safety reasons)
    let q = query!(Level1, "level2_id eq 2");
    assert!(toql.delete_many(q).await.is_ok());
    assert!(toql.sqls_empty());
}
