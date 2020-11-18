//! Trait to associate a field type provider with a struct.

use crate::query::wildcard::Wildcard;
use crate::query::selection::Selection;

/// Used by code produced from Toql derive.
pub trait QueryPath where Self: std::marker::Sized  {
    
     fn wildcard(self) -> Wildcard {
        Wildcard::from(self.into_path())
    }
     fn selection(self, name: &str) -> Selection {
        Selection::from(format!("{}{}",self.into_path(), name))
    }

     fn into_path(self) -> String;
    
}

