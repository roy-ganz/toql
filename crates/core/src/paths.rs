use std::marker::PhantomData;

pub struct Paths<T>{
    pub list: Vec<String>,
    marker: PhantomData<T>
}
 
impl<T> Paths< T> {
   
    pub fn root() -> Self {
        Self::from(vec![])
    }

    pub fn from(fields: Vec<String>) -> Self {
        Paths {
            list: fields,
            marker: PhantomData
        }
    }
} 