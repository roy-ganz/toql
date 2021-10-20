//! Traits to access and change the key of a Toql derived struct.

/// Trait to read the key of a struct.
///
/// Implemented automatically for every Toql derived struct.
///
/// The trait can be used by library users.
///
/// ### Example
/// Basic usage (assume a Toql derived User struct):
/// ```rust
/// use toql::prelude::Keyed;
/// let u = User{...};
/// let k = u.key();
/// ```
/// For collections there is [map_key](crate::map_key::map_key). It makes use of this trait.
pub trait Keyed {
    /// Type of key.
    type Key: Eq + std::hash::Hash + crate::key::Key;

    /// Return value of the key for a given entity.
    fn key(&self) -> Self::Key;
}

/// Trait to set the key of a `struct`.
///
/// Implemented automatically for every Toql derived struct.
/// The trait can be used by library users.
///
/// ### Example
/// Basic usage (assume a Toql derived User struct):
/// ```rust
/// use toql::prelude::Keyed;
/// let u = User{...};
/// let k = u.set_key(5.into());
/// ```
/// Here the number 5 is converted into the key type of `User`. Then this key is
/// set.
pub trait KeyedMut: Keyed {
    /// Set the key of the implementing `struct`.
    fn set_key(&mut self, key: Self::Key);
}
