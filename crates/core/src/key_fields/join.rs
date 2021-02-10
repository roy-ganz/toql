

use crate::{keyed::Keyed, key_fields::KeyFields, join::Join};

impl<T> KeyFields for Join<T>
where
    T: KeyFields + Keyed,
    <T as crate::keyed::Keyed>::Key: crate::key_fields::KeyFields
    
{
    type Entity = T::Entity;
    fn fields() -> Vec<String> {
        <T as KeyFields>::fields()
    }
    fn params(&self) -> Vec<crate::sql_arg::SqlArg>  {
        match self {
            Join::Key(k) => KeyFields::params(k),
            Join::Entity(e) => KeyFields::params(&e.key()),
        }
    }
    
   
}

