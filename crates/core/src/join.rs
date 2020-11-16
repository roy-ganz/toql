pub mod keyed;
pub mod tree_identity;
pub mod tree_merge;
pub mod tree_predicate;


#[derive(Debug, PartialEq, Eq)]
pub enum Join<E: crate::key::Keyed> {
    Key(E::Key),
    Entity(E),
}






/*
impl<K> Serialize for Join<K> {


}*/
