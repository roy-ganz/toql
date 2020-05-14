

// An trait to turn entity keys into a query perdicate (used by toql derive)
pub trait QueryPredicate<T> {
    fn predicate(self, path: &str) -> Query<T>;
}
