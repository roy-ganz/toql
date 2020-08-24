
trait PathPredicate {
    
    fn predicate_for_path(single_predicate: &str, path: &crate::field_path::FieldPath) -> crate::error::Result<crate::sql::Sql>


}