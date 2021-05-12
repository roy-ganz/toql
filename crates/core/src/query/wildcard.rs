/// A wildcard is used to select all fields from top or from a path.
///
/// Example
/// ```ignore
///
///  let q = Query::new().and(Wildcard::new()).and(Wildcard::from("bar")); // more elegant -> Query::wildcard().and(...)
///
///  assert_eq!("*, bar_*", q.to_string());
/// ```
/// Note that the Toql derive builds a wildcard function too.
/// If a struct `Foo` contained a struct `Bar`, it would be possible to replace the second call to _and()_ with  `.and(Bar::fields().bar().wildcard())`
///
use super::concatenation::Concatenation;

#[derive(Clone, Debug)]
pub struct Wildcard {
    pub(crate) concatenation: Concatenation,
    pub(crate) path: String,
}

impl Wildcard {
    /// Creates a new wildcard to select all fields from top
    pub fn new() -> Self {
        Wildcard {
            concatenation: Concatenation::And,
            path: String::from(""),
        }
    }
    /// Creates a new wildcard to select all fields from a path
    pub fn from<T>(path: T) -> Self
    where
        T: Into<String>,
    {
        let mut path = path.into();
        #[cfg(debug)]
        {
            if !path.chars().all(|x| x.is_alphanumeric() || x == '_') {
                panic!(
                    "Path {:?} must only contain alphanumeric characters and underscores.",
                    path
                );
            }
        }
        // Remove optional trailing *
        if path.ends_with('*') {
            path.pop();
        }
        // Add _ at end if missing
        if !path.ends_with('_') {
            path.push('_');
        }

        Wildcard {
            concatenation: Concatenation::And,
            path,
        }
    }

    pub fn into_string(self) -> String {
        self.path
    }
}

impl Default for Wildcard {
    fn default() -> Self {
        Self::new()
    }
}
