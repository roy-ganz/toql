#[derive(Debug, Clone)]
pub enum MergeColumn {
    Aliased(String),
    Unaliased(String),
}
#[derive(Debug, Clone)]
pub struct MergeMatch {
    pub other: MergeColumn,
    pub this: String,
}
#[derive(Debug, PartialEq)]
pub enum MergeSelection {
    // Option<T>
    Select,
    // T
    Preselect,
}

#[derive(Debug)]
pub struct MergeField {
    pub sql_join_table_name: String,
    pub join_alias: String,
    pub columns: Vec<MergeMatch>,
    pub join_sql: Option<String>,
    pub on_sql: Option<String>,
    pub selection: MergeSelection,
}
