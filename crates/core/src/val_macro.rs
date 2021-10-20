//! Convenience macros to work with the `Option` type.
//!
//! Use `val!` and `rval!`, if you expect `Option` to contain an value.
//! The macro will fail with a [ToqlError::NoneError](crate::error::ToqlError::NoneError), if `Option` is `None`. 
//!
//! ### Example 
//! With a field like `name: Option<String>` in an object `user` you can write 
//! ```rust
//! let address : &Address = rval!(user.address)?;
//! let address : Address = val!(user.address)?;
//! ```
//! 
//! The macros are useful, because typical Toql derive structs contain a lot of 
//! selectable fields, which are optional. The macros make it more convenenient to 
//! get the values out of those fields. 
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
