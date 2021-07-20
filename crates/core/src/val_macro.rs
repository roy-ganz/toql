

#[macro_export]
macro_rules! val {
    ($x: expr) => {
        $x.ok_or(toql::none_error!())?
    };
}
#[macro_export]
macro_rules! rval {
    ($x: expr) => {
        $x.as_ref().ok_or(toql::none_error!())?
    };
}
