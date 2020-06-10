
use super::concatenation::Concatenation;

#[derive(Clone, Debug)]
pub struct Selection {
    pub(crate) concatenation: Concatenation,
    pub(crate) name: String,
}


#[derive(Clone, Debug)]
pub struct SelectionPool<'a> {
    pub(crate) selections: &'a[&'a Selection],
}