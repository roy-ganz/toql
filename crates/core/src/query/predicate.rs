// A Toql predicate provides additional filtering.
/// A field can be created from a field name and filtered, sorted with its methods.
/// However the Toql derive creates fields structs for a derived struct, so instead of
/// ``` ignore
///  
///  let f = Predicate::from("search").is(["what"]);
/// ```
/// its easier and recommended to write
/// ``` ignore
///  let f = User::predicates().search(&["what"]);
/// ```
use super::concatenation::Concatenation;
use crate::sql_arg::SqlArg;

#[derive(Clone, Debug)]
pub struct Predicate {
    pub(crate) concatenation: Concatenation,
    pub(crate) name: String,
    pub(crate) args: Vec<SqlArg>,
}

impl Predicate {
    /// Create a field for the given name.
    pub fn from<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        let name = name.into();
        #[cfg(debug_assertions)]
        {
            if !name.chars().all(|x| x.is_alphanumeric() || x == '_') {
                panic!(
                    "Predicate {:?} must only contain alphanumeric characters and underscores.",
                    name
                );
            }
        }

        Predicate {
            concatenation: Concatenation::And,
            name,
            args: Vec::new(),
        }
    }

    /// Add single argument to predicate
    pub fn is(mut self, arg: impl Into<SqlArg>) -> Self {
        //self.args = arg.to_args();
        self.args.push(arg.into());
        self
    }
    // Add multiple Arguments
    pub fn are<I, T>(mut self, args: I) -> Self
    where
        T: Into<SqlArg>,
        I: IntoIterator<Item = T>,
    {
        for a in args {
            self.args.push(a.into());
        }
        self
    }
}

impl ToString for Predicate {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push('@');

        s.push_str(&self.name);
        s.push(' ');

        for a in &self.args {
            s.push_str(&a.to_string());
            s.push(' ');
        }

        s.pop();
        s
    }
}

impl From<&str> for Predicate {
    fn from(s: &str) -> Predicate {
        Predicate::from(s)
    }
}


#[cfg(test)]
mod test {
    use super::Predicate;

    #[test]
    fn build() {
        assert_eq!(Predicate::from("pred").to_string(), "@pred");
        assert_eq!(Predicate::from("pred").is(5).to_string(), "@pred 5");
        assert_eq!(Predicate::from("pred").are(vec![5, 10]).to_string(), "@pred 5 10");
    }
}