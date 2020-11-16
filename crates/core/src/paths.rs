use std::marker::PhantomData;

pub struct Paths<'a, T>{
    pub list: &'a [&'a str],
    marker: PhantomData<T>
}
 
impl<'a, T> Paths<'a, T> {
   
    pub const ROOT : &'a [&'static str] = &[];

    pub fn root() -> Self {
        Self::from(Self::ROOT)
    }

    pub fn from(fields: &'a [&'a str]) -> Self {
        Paths {
            list: fields,
            marker: PhantomData
        }
    }
       
} 