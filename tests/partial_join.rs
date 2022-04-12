use pretty_assertions::assert_eq;
use toql::mock_db::MockDb;
use toql::prelude::{fields, paths, query, Cache, Toql, ToqlApi};
use tracing_test::traced_test;

/*
#[derive(Debug, Default)]
pub struct Level1 {
    id: u64,
    text: String,
    level2: Option<Level2>, // Selectable inner join
}
#[derive(Debug, Default)]
pub struct Level2 {
    id: u64,
    text: String,
    level3: Option<Level3>, // Selectable inner join
}

#[derive(Debug, Default)]
pub struct Level3 {
    id: u64,
    text: String,
    level4: Option<Level4>, // Preselected left join
}
#[derive(Debug, Default)]
pub struct Level4 {
    id: u64,
    text: String,
}*/

#[derive(Debug, Default, Toql)]
#[toql(auto_key)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(join(partial_table))] // Partial join
    level2: Option<Level2>, // Selectable inner join
}
#[derive(Debug, Default, Toql)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(join(columns(self = "id", other = "id"), partial_table))]
    // Partial join (Cascaded partial join Level1, Level2, Level3)
    level3: Option<Level3>, // Selectable inner join
}

#[derive(Debug, Default, Toql)]
pub struct Level3 {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(join, preselect)]
    level4: Option<Level4>, // Preselected left join
}
#[derive(Debug, Default, Toql)]
pub struct Level4 {
    #[toql(key)]
    id: u64,
    text: String,
}

fn populated_level() -> Level1 {
    let l4 = Level4 {
        id: 4,
        text: "level4".to_string(),
    };
    let l3 = Level3 {
        id: 3,
        text: "level3".to_string(),
        level4: Some(l4),
    };
    let l2 = Level2 {
        id: 2,
        text: "level2".to_string(),
        level3: Some(l3),
    };

    Level1 {
        id: 1,
        text: "level1".to_string(),
        level2: Some(l2),
    }
}

#[tokio::test]
#[traced_test("info")]
async fn load() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load level 1 + preselected level 2
    let q = query!(Level1, "*");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1.text FROM Level1 level1"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn insert() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    let mut l = Level1::default();

    // insert level 1 and partial level 2 + 3 on default value
    assert!(toql.insert_one(&mut l, paths!(top)).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "INSERT INTO Level1 (text) VALUES ('')"
    );

    // insert level 1 and partial level 2 + 3 on populated value
    let mut l = populated_level();
    assert!(toql.insert_one(&mut l, paths!(top)).await.is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
        [
            "INSERT INTO Level1 (text) VALUES ('level1')",
            "INSERT INTO Level2 (id, text) VALUES (100, 'level2')",
            "INSERT INTO Level3 (id, text, level4_id) VALUES (100, 'level3', 4)"
        ]
    );

    // insert level 1, partial level 2 + 3, level4 on populated value
    let mut l = populated_level();
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3_level4"))
        .await
        .is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
        [
            "INSERT INTO Level4 (id, text) VALUES (4, 'level4')", // Regular joins come first
            "INSERT INTO Level1 (text) VALUES ('level1')", // Partial joins come last (top down)
            "INSERT INTO Level2 (id, text) VALUES (100, 'level2')",
            "INSERT INTO Level3 (id, text, level4_id) VALUES (100, 'level3', 4)",
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

    // Update level 1
    // Nothing is updated, fields are empty
    let mut l1 = Level1::default();
    assert!(toql
        .update_one(&mut l1, fields!(Level1, "level2_level3_*"))
        .await
        .is_ok());
    assert_eq!(toql.sqls_empty(), true);

    // Update level 1 with invalid key
    // Nothing is updated
    let mut l1 = populated_level();
    l1.id = 0;
    assert!(toql.update_one(&mut l1, fields!(top)).await.is_ok());
    assert_eq!(toql.sqls_empty(), true);

    // Update level 1 (text + foreign key is skipped, beause partial join shares primary key)
    let mut l1 = populated_level();
    assert!(toql.update_one(&mut l1, fields!(top)).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level1 SET text = 'level1' WHERE id = 1"
    );

    // Update level 4 (text)
    let mut l1 = populated_level();
    assert!(toql
        .update_one(&mut l1, fields!(Level1, "level2_level3_level4_*"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level4 SET text = 'level4' WHERE id = 4"
    );

    // Update level 1 - 4
    let mut l1 = populated_level();
    assert!(toql
        .update_one(
            &mut l1,
            fields!(
                Level1,
                "*, level2_*, \
            level2_level3_*, level2_level3_level4_*"
            ),
        )
        .await
        .is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
        [
            "UPDATE Level1 SET text = \'level1\' WHERE id = 1", // Partial join, no foreign key
            "UPDATE Level2 SET text = \'level2\' WHERE id = 1", // Partial join with same id as Level1 , no foreign key
            "UPDATE Level3 SET text = \'level3\', level4_id = 4 WHERE id = 1", // Partial join on Level 2 (Level 1) with  left join has foreign key
            "UPDATE Level4 SET text = \'level4\' WHERE id = 4"
        ]
    );
}
