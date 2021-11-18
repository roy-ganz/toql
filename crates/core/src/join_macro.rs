//! Convenience macros to work with the [Join](crate::join::Join) type.
//!
//! Use `join!`, if you expect a [Join](crate::join::Join) to contain an entity.
//! The macro will fail with a [ToqlError::NoneError](crate::error::ToqlError::NoneError), if the entity is missing.
//!
//! ### Example
//! With a join like `address: Join<Address>` in an object `user` you can write
//! ```rust, ignore
//! use toql_core::join;
//!
//! let address : &Address = join!(user.address).expect("Entity is missing.");
//! ```
//! Likewise for `Option<Join<Address>>` use `rval_join!`.

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
