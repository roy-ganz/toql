// A simple newtype to wrap a databse connection.
// Needed by select trait.
pub struct Conn<T>(T);
