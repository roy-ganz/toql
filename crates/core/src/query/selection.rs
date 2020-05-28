
use super::concatenation::Concatenation;

#[derive(Clone, Debug)]
pub struct Selection {
    pub(crate) concatenation: Concatenation,
    pub(crate) name: String,
}