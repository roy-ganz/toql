use pretty_assertions::assert_eq;
use toql::mock_db::MockDb;
use toql::prelude::{query, Cache, Join, Page, Toql, ToqlApi};
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
#[toql(
    predicate(name = "pred", sql = "..text = ?", count_filter),
    selection(name = "cnt", fields = "text")
)]
pub struct Level1 {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(join)] // Default mapping
    level2: Option<Option<Join<Level2>>>, // Preselected inner join
}

#[derive(Debug, Default, Toql)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    text: String,
}

#[tokio::test]
#[traced_test("info")]
async fn load_page() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache); // MockDb does not really issue SQL for page limitation

    // Load level 2 + count query
    let q = query!(Level1, "level2_text, id eq 5");
    let page = Page::Counted(1, 10);
    assert!(toql.load_page(q, page).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sqls(),
        [
            "SELECT level1.id, level1.text, level1_level2.id, level1_level2.text \
            FROM Level1 level1 \
            LEFT JOIN (Level2 level1_level2) ON (level1.level2_id = level1_level2.id) \
            WHERE level1.id = 5",
            "",
            "SELECT COUNT(*) FROM Level1 level1"
        ]
    );

    // Load level2 with filter on count field
    let q = query!(Level1, "level2_text, text eq 'ABC'");
    let page = Page::Counted(1, 10);
    assert!(toql.load_page(q, page).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sqls(),
        [
            "SELECT level1.id, level1.text, level1_level2.id, level1_level2.text \
            FROM Level1 level1 \
            LEFT JOIN (Level2 level1_level2) ON (level1.level2_id = level1_level2.id) \
            WHERE level1.text = 'ABC'",
            "",
            "SELECT COUNT(*) FROM Level1 level1 WHERE level1.text = 'ABC'"
        ]
    );

    // Load level2 with predicate filter on count field
    let q = query!(Level1, "level2_text, @pred 'ABC'");
    let page = Page::Counted(1, 10);
    assert!(toql.load_page(q, page).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sqls(),
        [
            "SELECT level1.id, level1.text, level1_level2.id, level1_level2.text \
            FROM Level1 level1 \
            LEFT JOIN (Level2 level1_level2) ON (level1.level2_id = level1_level2.id) \
            WHERE level1.text = 'ABC'",
            "",
            "SELECT COUNT(*) FROM Level1 level1 WHERE level1.text = 'ABC'"
        ]
    );
}
