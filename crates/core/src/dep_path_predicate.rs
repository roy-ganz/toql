
trait PathPredicate {
    
    fn predicate_for_path<'a>(&self, single_predicate: &str, descendents: &crate::query::field_path::Descendents<'a>) -> crate::error::Result<crate::sql::Sql>;


}