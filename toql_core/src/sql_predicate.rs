
/// Trait to turn a partial or full key into a sql predicate
pub trait SqlPredicate {
    type Entity; // Output type

    /// Returns SQL predicate
    fn sql_predicate(&self, alias:&str) -> (String, Vec<String>);

    /// Returns SQL predicate for collection. 
    /// This may be overridded for simple primary keys that are build with IN(..)
    fn sql_any_predicate(predicates: &[Self], alias:&str) -> (String, Vec<String>) 
    where Self:Sized
    {
        let mut predicate = String::new();
         let mut params = Vec::new();

         for p in predicates {
             let (pr, pa) = p.sql_predicate(alias);
             predicate.push_str(&pr);
             params.extend_from_slice(&pa);
             predicate.push_str(" OR ")
         }

         predicate.pop();
         predicate.pop();
         predicate.pop();
         predicate.pop();

         (predicate, params)
     }

}

/* 
impl<T, U> SqlPredicate for &[U] 
where U: SqlPredicate<Entity =T>
{
    type Entity= T;

     fn sql_predicate(&self, alias:&str) -> (String, Vec<String>){

         let mut predicate = String::new();
         let mut params = Vec::new();

         for i in *self {
             let (pr, pa) = i.sql_predicate(alias);
             predicate.push_str(&pr);
             params.extend_from_slice(&pa);
             predicate.push_str(" OR ")
         }

         predicate.pop();
         predicate.pop();
         predicate.pop();
         predicate.pop();

         (predicate, params)
     }
}
 */