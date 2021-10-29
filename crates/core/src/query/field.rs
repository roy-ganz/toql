/// A Toql field can select, filter and order a database column or expression
/// A field can be created from a field name and filtered, sorted with its methods.
/// However the Toql derive creates fields structs for a derived struct, so instead of
/// ``` ignore
///  
///  let f = Field::from("id");
/// ```
/// its easier and recommended to write
/// ``` ignore
///  let f = User::fields().id();
/// ```
use super::concatenation::Concatenation;
use super::field_filter::FieldFilter;
use super::field_order::FieldOrder;
use crate::sql_arg::SqlArg;
//use heck::MixedCase;

#[derive(Clone, Debug)]
pub struct Field {
    pub(crate) concatenation: Concatenation,
    pub(crate) name: String,
    pub(crate) hidden: bool,
    pub(crate) order: Option<FieldOrder>,
    pub(crate) filter: Option<FieldFilter>,
  //  pub(crate) aggregation: bool,
}

impl Field {
    /// Create a field for the given name.
    pub fn from<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        let name = name.into();
        #[cfg(debug_assertions)]
        {
            // Ensure name does not end with wildcard
            if !name.chars().all(|x| x.is_alphanumeric() || x == '_') {
                panic!(
                    "Field {:?} must only contain alphanumeric characters and underscores.",
                    name
                );
            }
        }

        Field {
            concatenation: Concatenation::And,
            name,
            hidden: false,
            order: None,
            filter: None,
           // aggregation: false,
        }
    }
  /*   pub fn canonical_alias(&self, root: &str) -> String {
        format!("{}_{}", root.to_mixed_case(), self.name)
    } */

    /// Hide field. Useful if a field should not be selected, but be used for filtering.
    pub fn hide(mut self) -> Self {
        self.hidden = true;
        self
    }
   /*  /// Aggregate a field to make the filter be in SQL HAVING clause instead of WHERE clause
    pub fn aggregate(mut self) -> Self {
        self.aggregation = true;
        self
    } */
    /// Use this field to order records in ascending way. Give ordering priority when records are ordered by multiple fields.
    pub fn asc(mut self, order: u8) -> Self {
        self.order = Some(FieldOrder::Asc(order));
        self
    }
    /// Use this field to order records in descending way. Give ordering priority when records are ordered by multiple fields.
    pub fn desc(mut self, order: u8) -> Self {
        self.order = Some(FieldOrder::Desc(order));
        self
    }
    /// Filter records with _equal_ predicate.
    pub fn eq(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Eq(criteria.into()));
        self
    }
    /// Filter records with _equal null_ predicate.
    pub fn eqn(mut self) -> Self {
        self.filter = Some(FieldFilter::Eqn);
        self
    }
    /// Filter records with _not equal_ predicate.
    pub fn ne(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Ne(criteria.into()));
        self
    }
    /// Filter records with _not equal null_ predicate.
    pub fn nen(mut self) -> Self {
        self.filter = Some(FieldFilter::Nen);
        self
    }
    /// Filter records with greater that_ predicate.
    pub fn gt(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Gt(criteria.into()));
        self
    }
    /// Filter records with greater or equal_ predicate.
    pub fn ge(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Ge(criteria.into()));
        self
    }
    /// Filter records with lesser than_ predicate.
    pub fn lt(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Lt(criteria.into()));
        self
    }
    /// Filter records with lesser or equal_ predicate.
    pub fn le(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Le(criteria.into()));
        self
    }
    /// Filter records with _between_ predicate. This is inclusive, so `x bw 3 6` is the same as `x ge 3, x le 6`
    pub fn bw(mut self, lower: impl Into<SqlArg>, upper: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Bw(lower.into(), upper.into()));
        self
    }
    /// Filter records with _like_ predicate.
    pub fn lk(mut self, criteria: impl Into<SqlArg>) -> Self {
        self.filter = Some(FieldFilter::Lk(criteria.into()));
        self
    }
    /// Filter records with _inside_ predicate.
    pub fn ins<T, I>(mut self, criteria: I) -> Self
    where
        T: Into<SqlArg>,
        I: IntoIterator<Item = T>,
    {
        self.filter = Some(FieldFilter::In(
            criteria.into_iter().map(|c| c.into()).collect(),
        ));
        self
    }
    /// Filter records with _outside_ predicate.
    pub fn out<T, I>(mut self, criteria: I) -> Self
    where
        T: Into<SqlArg>,
        I: IntoIterator<Item = T>,
    {
        self.filter = Some(FieldFilter::Out(
            criteria.into_iter().map(|c| c.into()).collect(),
        ));
        self
    }
    /// Filter records with custom function.
    /// To provide a custom function you must implement (FieldHandler)[../table_mapper/trait.FieldHandler.html]
    /// See _custom handler test_ for an example.
    pub fn fnc<U, T, I>(mut self, name: U, args: I) -> Self
    where
        U: Into<String>,
        T: Into<SqlArg>,
        I: IntoIterator<Item = T>,
    {
        self.filter = Some(FieldFilter::Fn(
            name.into(),
            args.into_iter().map(|c| c.into()).collect(),
        ));
        self
    }
   
    pub fn concatenate(mut self, concatenation: Concatenation) -> Self {
        self.concatenation = concatenation;
        self
    }

    pub fn into_name(self) -> String {
        self.name
    }
}

impl ToString for Field {
    fn to_string(&self) -> String {
        let mut s = String::new();
        match self.order {
            Some(FieldOrder::Asc(o)) => {
                s.push('+');
                s.push_str(&o.to_string());
            }
            Some(FieldOrder::Desc(o)) => {
                s.push('-');
                s.push_str(&o.to_string());
            }
            None => {}
        };
        if self.hidden {
            s.push('.');
        }
        s.push_str(&self.name);

        if self.filter.is_some() {
          /*   if self.aggregation {
                s.push_str(" !");
            } else { */
                s.push(' ');
            //}
        }
        match self.filter {
            None => {}
            Some(ref filter) => {
                s.push_str(filter.to_string().as_str());
            }
        }
        s
    }
}

/* impl From<&str> for Field {
    fn from(s: &str) -> Field {
        Field::from(s)
    }
} */


#[cfg(test)]
mod test {
    use super::Field;

    #[test]
    fn build() {
        assert_eq!(Field::from("prop").eq(true).to_string(), "prop EQ 1");
        assert_eq!(Field::from("prop").eqn().to_string(), "prop EQN");
        assert_eq!(Field::from("prop").ne(1).to_string(), "prop NE 1");
        assert_eq!(Field::from("prop").nen().to_string(), "prop NEN");
        assert_eq!(Field::from("prop").gt(1).to_string(), "prop GT 1");
        assert_eq!(Field::from("prop").ge(1.5).to_string(), "prop GE 1.5");
        assert_eq!(Field::from("prop").lt(1.5).to_string(), "prop LT 1.5");
        assert_eq!(Field::from("prop").le(1).to_string(), "prop LE 1");
        assert_eq!(Field::from("prop").lk("%ABC%").to_string(), "prop LK '%ABC%'");
        assert_eq!(Field::from("prop").bw(1, 10).to_string(), "prop BW 1 10");
        assert_eq!(Field::from("prop").ins(vec![1, 10]).to_string(), "prop IN 1 10");
        assert_eq!(Field::from("prop").out(vec![1, 10]).to_string(), "prop OUT 1 10");
        assert_eq!(Field::from("prop").fnc("SC", vec![1, 10]).to_string(), "prop FN SC 1 10");

        assert_eq!(Field::from("prop").asc(1).to_string(), "+1prop");
        assert_eq!(Field::from("prop").desc(3).to_string(), "-3prop");
        assert_eq!(Field::from("prop").hide().to_string(), ".prop");

        // Combination
        assert_eq!(Field::from("level3_prop").eq(10).hide().asc(4).to_string(), "+4.level3_prop EQ 10");
    }

     #[test]
     fn into_name() {
          assert_eq!(Field::from("level3_prop").eq(10).hide().asc(4).into_name(), "level3_prop");
     }
     
     #[test]
     #[should_panic]
     fn invalid_name() {
         Field::from("level%2");
     }
    
}