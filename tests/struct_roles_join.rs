use std::collections::HashSet;

use pretty_assertions::assert_eq;
use toql::mock_db::MockDb;
use toql::prelude::{fields, paths, query, Cache, ContextBuilder, Join, Toql, ToqlApi};
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

    #[toql(join)]
    level2: Option<Join<Level2>>,
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
    text: Option<String>, // Selectable
}

fn populated_level() -> Level1 {
    let l2 = Level2 {
        id: 2,
        text: Some("level2".to_string()),
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
    // fail to load fields
    let q = query!(Level1, "*");
    assert!(toql.load_many(q).await.is_err());

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
    let q = query!(Level1, "level2_*");
    assert!(toql.load_many(q).await.is_err());

    let mut toql = toql_for_roles(&["load1_role", "load2_role"], &cache);
    let q = query!(Level1, "level2_*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id, level1_level2.text \
        FROM Level1 level1 \
        JOIN (Level2 level1_level2) ON (level1.level2_id = level1_level2.id)"
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
    assert!(toql.update_one(&mut l, fields!(Level1, "*")).await.is_err());

    // Update level 1 text with role
    let mut toql = toql_for_roles(&["upd1_role"], &cache);
    assert!(toql.update_one(&mut l, fields!(Level1, "*")).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level1 SET text = \'level1\', level2_id = 2 WHERE id = 1"
    );

    // Update level 2: Role for level2, role for leve1 is missing
    // That's fine, update fields from level2
    let mut toql = toql_for_roles(&["upd2_role"], &cache);
    assert!(toql
        .update_one(&mut l, fields!(Level1, "level2_*"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level2 SET text = \'level2\' WHERE id = 2"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn insert() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    let mut l = populated_level();

    // Insert level 1: No Role
    // Throws errror
    assert!(toql.insert_one(&mut l, paths!(Level1, "")).await.is_err());

    // Insert level 1 with role
    let mut toql = toql_for_roles(&["ins1_role"], &cache);
    assert!(toql.insert_one(&mut l, paths!(Level1, "")).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "INSERT INTO Level1 (text, level2_id) VALUES (\'level1\', 2)"
    );

    // Insert level 2 with missing role for level1
    let mut toql = toql_for_roles(&["ins2_role"], &cache);
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2"))
        .await
        .is_err());

    // Insert level 2 with roles
    let mut toql = toql_for_roles(&["ins1_role", "ins2_role"], &cache);
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2"))
        .await
        .is_ok());

    assert_eq!(
        toql.take_unsafe_sqls(),
        [
            "INSERT INTO Level1 (text, level2_id) VALUES (\'level1\', 2)",
            "INSERT INTO Level2 (text) VALUES (\'level2\')"
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
    assert!(toql.delete_many(q).await.is_err());

    // Delete Level1 with role
    let mut toql = toql_for_roles(&["del1_role"], &cache);
    let q = query!(Level1, "id eq 1");
    assert!(toql.delete_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "DELETE level1 FROM Level1 level1 WHERE level1.id = 1"
    );

    // Delete level 1 with level 2 join and role for level1
    // Succeeds
    let q = query!(Level1, "level2_id eq 2");
    assert!(toql.delete_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "DELETE level1 FROM Level1 level1 JOIN (Level2 level1_level2) ON (level1.level2_id = level1_level2.id) WHERE level1_level2.id = 2"
    );
}
