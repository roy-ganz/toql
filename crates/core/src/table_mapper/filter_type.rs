
#[derive(Debug)]
#[allow(dead_code)] // IMPROVE Having AND None are considered unused
pub(crate) enum FilterType {
    Where,
    Having,
    None,
}