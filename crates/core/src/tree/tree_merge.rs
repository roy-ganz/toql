use crate::query::field_path::Descendents;

pub trait TreeMerge {
    fn merge<'a, 'b, R, E>(&self, descendents: &Descendents<'a>, row: &'b R) -> Result<(), E>;
}
