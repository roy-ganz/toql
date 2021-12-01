use std::collections::HashSet;

use pretty_assertions::assert_eq;
use toql::mock_db::MockDb;
use toql::prelude::{
    fields, query, Cache, ContextBuilder, SqlBuilderError, Toql, ToqlApi, ToqlError,
};
use toql::row;
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
#[toql(auto_key)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    #[toql(roles(load = "load1_role", update = "update1_text_role"))]
    text: Option<String>,

    #[toql(merge, roles(load = "load2_role", update = "update1_level2_role"))]
    level2: Option<Vec<Level2>>,
}
#[derive(Debug, Default, Toql)]
#[toql(auto_key)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    #[toql(foreign_key)]
    level1_id: u64,
    #[toql(roles(load = "load2_text_role", update = "update2_text_role"))]
    text: Option<String>, // Selectable

    #[toql(merge, roles(load = "load2_level3_role"))]
    level3: Vec<Level3>,
}

#[derive(Debug, Default, Toql)]
#[toql(auto_key)]
pub struct Level3 {
    #[toql(key)]
    id: u64,
    #[toql(foreign_key)]
    level2_id: u64,
    #[toql(roles(load = "load3_text_role"))]
    text: String, // Preselected
}

fn populated_level() -> Level1 {
    let l3 = Level3 {
        id: 0,
        level2_id: 2,
        text: "level3".to_string(),
    };
    let l2 = Level2 {
        id: 2,
        level1_id: 1,
        text: Some("level2".to_string()),
        level3: vec![l3], // New item
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

    // Load level 1 : No role
    // -> only id is loaded,
    // -> Selectable join and merge do not required roles
    let q = query!(Level1, "*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id FROM Level1 level1"
    );

    // Load level 1 : No role
    // -> fail to load text
    let q = query!(Level1, "text");
    let err = toql.load_many(q).await.err().unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlBuilderError(SqlBuilderError::RoleRequired(
            "load1_role".to_string(),
            "field `text`".to_string()
        ))
        .to_string()
    );

    // Load level 1: load1_role
    // -> only id, text is loaded
    let mut toql = toql_for_roles(&["load1_role"], &cache);
    let q = query!(Level1, "*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text FROM Level1 level1"
    );
    // Load level 2: missing role `load2_role`
    // -> fail to load join
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

    // Load level 1, 2: preselected merge `level3` is missing role
    // Succeeds, because no entities are loaded for level1
    let mut toql = toql_for_roles(&["load2_role"], &cache);
    let q = query!(Level1, "level2_*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id FROM Level1 level1"
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
    toql.mock_rows("SELECT level1.id FROM Level1 level1", vec![row!(1u64)]);
    toql.mock_rows(
        "SELECT level1.id, level2.id, level2.level1_id \
                        FROM Level2 level2 \
                        JOIN Level1 level1 ON (level1.id = level2.level1_id AND level1.id = 1)",
        vec![row!(1u64, 2u64, 1u64)],
    );
    toql.mock_rows("SELECT level1_level2.id, level3.id, level3.level2_id, level3.text \
                        FROM Level3 level3 \
                        JOIN Level2 level1_level2 ON (level1_level2.id = level3.level2_id AND level1_level2.id = 2)", 
                        vec![row!(2u64, 3u64, 2u64, "level3")]);
    let q = query!(Level1, "level2_level3_*");
    assert!(toql.load_many(q).await.is_ok());

    assert_eq!(
        toql.take_unsafe_sqls(),
        ["SELECT level1.id FROM Level1 level1",
        "SELECT level1.id, level2.id, level2.level1_id FROM Level2 level2 \
            JOIN Level1 level1 ON (level1.id = level2.level1_id AND level1.id = 1)",
        "SELECT level1_level2.id, level3.id, level3.level2_id, level3.text \
                        FROM Level3 level3 \
                        JOIN Level2 level1_level2 ON (level1_level2.id = level3.level2_id AND level1_level2.id = 2)"              
        ]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn load2() {
    let cache = Cache::new();

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
    let select1 = "SELECT level1.id FROM Level1 level1";
    let select2 = "SELECT level1.id, level2.id, level2.level1_id \
                    FROM Level2 level2 \
                    JOIN Level1 level1 \
                    ON (level1.id = level2.level1_id AND level1.id = 1) \
                    WHERE level2_level2.text = 'ABC' \
                    ORDER BY level2.text ASC";
    let select3 = "SELECT level1_level2.id, level3.id, level3.level2_id, level3.text \
                    FROM Level3 level3 \
                    JOIN Level2 level1_level2 \
                    ON (level1_level2.id = level3.level2_id AND level1_level2.id = 2)";

    toql.mock_rows(select1, vec![row!(1u64)]);
    toql.mock_rows(select2, vec![row!(1u64, 2u64, 1u64)]);
    toql.mock_rows(select3, vec![row!(2u64, 3u64, 2u64, "level3")]); // preselects on level2
    let q = query!(Level1, "+.level2_text eq 'ABC'");

    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(toql.take_unsafe_sqls(), [select1, select2, select3]);
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

    // Update level 1 text with role and resize vec level2
    let mut toql = toql_for_roles(&["update1_text_role", "update1_level2_role"], &cache);
    assert!(toql
        .update_one(&mut l, fields!(Level1, "*, level2"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sqls(),
        [
            "UPDATE Level1 SET text = 'level1' WHERE id = 1",
            "DELETE level1_level2 FROM Level2 level1_level2 \
            JOIN Level1 level1 ON level1.id = level1_level2.level1_id \
            WHERE level1.id = 1 AND NOT (level1_level2.id = 2)"
        ]
    );
}
