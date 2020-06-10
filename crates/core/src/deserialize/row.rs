
use crate::error::Result;


// Implemented by database support for every entity
// Reads a and deserializes an entity from a table row
// Returns the entity or throws an error
pub trait Row<T> {
    fn read(&mut self) ->Result<T>;
}