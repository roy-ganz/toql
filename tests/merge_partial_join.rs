use toql::mock_db::MockDb;
use toql::prelude::{fields, paths, query, Cache, Join, Toql, ToqlApi};
use toql::row;
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
#[toql(auto_key)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    text: Option<String>,

    #[toql(merge)]
    level2: Option<Vec<Level2>>, // Selectable merge
}
#[derive(Debug, Default, Toql)]
#[toql(auto_key)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    #[toql(foreign_key)]
    level1_id: u64,
    text: Option<String>,

    #[toql(join(partial_table))]
    level3: Option<Join<Level3>>, // Selectable inner join
}

#[derive(Debug, Default, Toql)]
pub struct Level3 {
    #[toql(key)]
    id: u64,
    text: Option<String>,
}

fn populated_level() -> Level1 {
    let l32 = Level3 {
        id: 22, // Partial joins share primary keys
        text: Some("new level32 (22)".to_string()),
    };
    let l22 = Level2 {
        id: 22,
        level1_id: 0,
        text: Some("new level22".to_string()),
        level3: Some(Join::with_entity(l32)),
    };
    let l31 = Level3 {
        id: 21, // Partial joins share primary keys
        text: Some("level31 (21)".to_string()),
    };
    let l21 = Level2 {
        id: 21,
        level1_id: 1,
        text: Some("level2".to_string()),
        level3: Some(Join::with_entity(l31)),
    };

    Level1 {
        id: 1,
        text: Some("level1".to_string()),
        level2: Some(vec![l21, l22]),
    }
}

#[tokio::test]
#[traced_test("info")]
async fn load() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load level 3
    let select1 = "SELECT level1.id FROM Level1 level1";
    let select2 =
        "SELECT level1.id, level2.id, level2.level1_id, level2_level3.id, level2_level3.text \
                    FROM Level2 level2 \
                    JOIN (Level3 level2_level3) ON (level2.level3_id = level2_level3.id) \
                    JOIN Level1 level1 ON (level1.id = level2.level1_id AND level1.id = 1)";

    let q = query!(Level1, "level2_level3_*");
    toql.mock_rows(select1, vec![row!(1u64)]);
    toql.mock_rows(select2, vec![row!(1u64, 21u64, 1u64, 21u64, "level3")]);
    assert!(toql.load_many(q).await.is_ok());

    assert_eq!(toql.take_unsafe_sqls(), [select1, select2]);
}

#[tokio::test]
#[traced_test("info")]
async fn insert() {
    let cache = Cache::default();
    let mut toql = MockDb::from(&cache);

    let mut l = Level1::default();

    // insert level 1..3
    // Will only insert level1 because left join is None
    // Does not insert id, because auto_key
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "INSERT INTO Level1 (text) VALUES (DEFAULT)"
    );

    // insert path levels 1..3
    // this will insert level 1..3
    let mut l = populated_level();
    assert!(toql
        .insert_one(&mut l, paths!(Level1, "level2_level3"))
        .await
        .is_ok());

    assert_eq!(
        toql.take_unsafe_sqls(),
        [
            "INSERT INTO Level1 (text) VALUES ('level1')",
            "INSERT INTO Level2 (level1_id, text) VALUES (100, 'level2'), (100, 'new level22')",
            "INSERT INTO Level3 (id, text) VALUES (102, 'level31 (21)'), (101, 'new level32 (22)')"
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

    // Update level 1 (text)
    let mut l1 = populated_level();
    assert!(toql.update_one(&mut l1, fields!(top)).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "UPDATE Level1 SET text = 'level1' WHERE id = 1"
    );

    // Update level 3 (text)
    // This ignores new item in Vec
    let mut l1 = populated_level();
    assert!(toql
        .update_one(&mut l1, fields!(Level1, "level2_level3_*"))
        .await
        .is_ok());
    assert_eq!(
        toql.take_unsafe_sqls(),
        [
            "UPDATE Level3 SET text = \'level31 (21)\' WHERE id = 21",
            "UPDATE Level3 SET text = \'new level32 (22)\' WHERE id = 22"
        ],
    );

    // Update level 1 - 3
    let mut l1 = populated_level();
    assert!(toql
        .update_one(
            &mut l1,
            fields!(
                Level1,
                "*, level2_*, \
            level2_level3_*"
            ),
        )
        .await
        .is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
        [
            "UPDATE Level1 SET text = 'level1' WHERE id = 1",
            "UPDATE Level2 SET level1_id = 1, text = 'level2' WHERE id = 21",
            "UPDATE Level2 SET level1_id = 1, text = 'new level22' WHERE id = 22",
            "UPDATE Level3 SET text = 'level31 (21)' WHERE id = 21",
            "UPDATE Level3 SET text = 'new level32 (22)' WHERE id = 22",
        ]
    );
}

#[tokio::test]
#[traced_test("info")]
async fn update2() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // resize vec of level2
    let mut l1 = populated_level();
    assert!(toql
        .update_one(&mut l1, fields!(Level1, "level2"))
        .await
        .is_ok());
    let sqls = toql.take_unsafe_sqls();
    assert_eq!(
        sqls,
        ["DELETE level1_level2 FROM Level2 level1_level2 \
        JOIN Level1 level1 ON level1.id = level1_level2.level1_id \
        WHERE level1.id = 1 AND NOT (level1_level2.id IN (21, 22))"]
    );
}
