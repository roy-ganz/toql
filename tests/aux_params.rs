use std::collections::HashMap;
use toql::{
    mock_db::MockDb,
    prelude::{
        query, Cache, ContextBuilder, FieldFilter, FieldHandler, Join, ParameterMap, Resolver,
        SqlArg, SqlBuilderError, SqlExpr, Toql, ToqlApi,
    },
};
use tracing_test::traced_test;

#[derive(Debug, Default, Toql)]
#[toql(predicate(
    name = "pred",
    sql = "..text1 = ?",
    on_aux_param(name = "on_param", index = 0)
))] // Inject into join predicate
pub struct Level1 {
    #[toql(key)]
    id: u64,
    #[toql(sql = "(SELECT <text1>)")]
    text1: Option<String>,

    #[toql(sql = "(SELECT <text2>)", aux_param(name = "text2", value = "hello2"))]
    // Local aux param
    text2: Option<String>,

    #[toql(
        sql = "(SELECT <text3>)",
        aux_param(name = "TEXT_3", value = "hello3"),
        handler = "get_field_handler"
    )] // Use handler to translate
    text3: Option<String>,

    #[toql(join(on_sql = "...text = <on_param>"))]
    level2: Option<Option<Join<Level2>>>, // Left join that depends on aux param
}

#[derive(Debug, Default, Toql)]
pub struct Level2 {
    #[toql(key)]
    id: u64,
    text: Option<String>,
}

struct MyFieldHandler;
impl FieldHandler for MyFieldHandler {
    fn build_select(
        &self,
        select: SqlExpr,
        aux_params: &ParameterMap<'_>,
    ) -> Result<Option<SqlExpr>, SqlBuilderError> {
        let value: &str = aux_params
            .get("TEXT_3")
            .map(|a| a.get_str().unwrap_or_default())
            .unwrap_or_default();
        let mut ap = HashMap::new();
        ap.insert("text3".to_string(), SqlArg::from(value));
        let ar = [&ap];
        let parameter_map = ParameterMap::new(&ar);
        let select = Resolver::resolve_aux_params(select, &parameter_map);
        Ok(Some(select))
    }
    fn build_filter(
        &self,
        select: SqlExpr,
        filter: &FieldFilter,
        aux_params: &ParameterMap<'_>,
    ) -> Result<Option<SqlExpr>, SqlBuilderError> {
        let f = toql::prelude::DefaultFieldHandler::default();
        f.build_filter(select, filter, aux_params)
    }
}
fn get_field_handler() -> MyFieldHandler {
    MyFieldHandler
}

#[tokio::test]
#[traced_test("info")]
async fn query_aux_params() {
    use toql::prelude::{ResolverError, ToqlError};

    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load text1 without aux param
    // -> Fails
    let q = query!(Level1, "text1");
    let err = toql.load_many(&q).await.err().unwrap();
    assert_eq!(
        err.to_string(),
        ToqlError::SqlExprResolverError(ResolverError::AuxParamMissing("text1".to_string()))
            .to_string()
    );

    // Load text1 with aux param
    let q = query!(Level1, "text1").aux_param("text1", "hello1");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, (SELECT 'hello1') FROM Level1 level1"
    );

    // Test with filter
    let q = query!(Level1, "text1 eq 'ABC'").aux_param("text1", "hello1");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, (SELECT 'hello1') FROM Level1 level1 WHERE (SELECT 'hello1') = 'ABC'"
    );

    // Test with order
    let q = query!(Level1, "+text1").aux_param("text1", "hello1");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, (SELECT 'hello1') FROM Level1 level1 ORDER BY (SELECT 'hello1') ASC"
    );

    // Load text2 with local aux param
    let q = query!(Level1, "text2");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, (SELECT 'hello2') FROM Level1 level1"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn context_aux_params() {
    let cache = Cache::new();
    let mut aux_params = HashMap::new();
    aux_params.insert("text1".to_string(), SqlArg::from("hello1"));
    let context = ContextBuilder::default()
        .with_aux_params(aux_params)
        .build();

    let mut toql = MockDb::with_context(&cache, context);
    let q = query!(Level1, "text1");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, (SELECT 'hello1') FROM Level1 level1"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn local_aux_params() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load text3 with field handler translation
    let q = query!(Level1, "text3");

    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, (SELECT 'hello3') FROM Level1 level1"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn wildcard() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load all fields, skip fields with missing aux params
    // -> text1 is skipped
    let q = query!(Level1, "*");

    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, (SELECT \'hello2\'), (SELECT \'hello3\') FROM Level1 level1"
    );
}

#[tokio::test]
#[traced_test("info")]
async fn left_join() {
    let cache = Cache::new();
    let mut toql = MockDb::from(&cache);

    // Load left join with missing aux param in on clause
    // Left join is disabled
    let q = query!(Level1, "level2_text");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id, level1_level2.text FROM Level1 level1 LEFT JOIN (Level2 level1_level2) ON (false)"
    );
    // Load left join with aux param in on clause through predicate
    let q = query!(Level1, "level2_text").aux_param("on_param", "explicit");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id, level1_level2.text \
            FROM Level1 level1 \
            LEFT JOIN (Level2 level1_level2) \
            ON (level1.level2_id = level1_level2.id AND level1_level2.text = \'explicit\')"
    );

    // Load left join with aux param in on clause through predicate
    let q = query!(Level1, "level2_text, @pred 'predicate'");
    assert!(toql.load_many(q).await.is_ok());
    assert_eq!(
        toql.take_unsafe_sql(),
        "SELECT level1.id, level1_level2.id, level1_level2.text \
            FROM Level1 level1 \
            LEFT JOIN (Level2 level1_level2) \
            ON (level1.level2_id = level1_level2.id AND level1_level2.text = \'predicate\') \
            WHERE level1.text1 = \'predicate\'"
    );
}
