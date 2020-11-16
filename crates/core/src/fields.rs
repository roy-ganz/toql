use std::marker::PhantomData;

pub struct Fields<'a, T>{
    pub list: &'a [&'a str],
    marker: PhantomData<T>
}
 
impl<'a, T> Fields<'a, T> {
   
    pub const WILDCARD : &'a [&'static str] = &["*"];

    pub fn wildcard() -> Self {
        Self::from(Self::WILDCARD)
    }

    pub fn from(fields: &'a [&'a str]) -> Self {
        Fields {
            list: fields,
            marker: PhantomData
        }
    }
       
} 