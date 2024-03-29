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
        // Remove optional trailing *
        if path.ends_with('*') {
            path.pop();
        }

        #[cfg(debug_assertions)]
        {
            if !path.chars().all(|x| x.is_alphanumeric() || x == '_') {
                panic!(
                    "Path {:?} must only contain alphanumeric characters and underscores.",
                    path
                );
            }
        }

        // Add _ at end if missing
        if !path.is_empty() && !path.ends_with('_') {
            path.push('_');
        }

        Wildcard {
            concatenation: Concatenation::And,
            path,
        }
    }

    pub fn into_string(self) -> String {
        format!("{}*", self.path)
    }
    pub fn into_path(self) -> String {
        self.path
    }
}

impl Default for Wildcard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::Wildcard;

    #[test]
    fn build() {
        assert_eq!(Wildcard::new().into_string(), "*");
        assert_eq!(Wildcard::default().into_string(), "*");
        assert_eq!(Wildcard::from("").into_string(), "*");
        assert_eq!(Wildcard::from("level2").into_string(), "level2_*");
        assert_eq!(Wildcard::from("level2_").into_string(), "level2_*"); // tolerate underscore
        assert_eq!(Wildcard::from("level2_*").into_string(), "level2_*"); // tolerate underscore and asterix

        assert_eq!(Wildcard::from("").into_path(), "");
        assert_eq!(Wildcard::from("level2").into_path(), "level2_"); // Path adds underscore
        assert_eq!(Wildcard::from("level2_").into_path(), "level2_");
    }
    #[test]
    #[should_panic]
    fn invalid_name() {
        Wildcard::from("level%2");
    }
}
