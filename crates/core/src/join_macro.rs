// Join returns always a reference
#[macro_export]
macro_rules! join {
    ($x: expr) => {
        toql::prelude::Join::entity_or_err($x, toql::none_error!())?
    };
}
#[macro_export]
macro_rules! rval_join {
    ($x: expr) => {
        toql::prelude::Join::entity_or_err(
            $x.as_ref().ok_or(toql::none_error!())?,
            toql::none_error!(),
        )?
    };
}
