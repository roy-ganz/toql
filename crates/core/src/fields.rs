use std::marker::PhantomData;

pub struct Fields<T>{
    pub list: Vec<String>,
    marker: PhantomData<T>
}
 
impl< T> Fields< T> {
   
    

    pub fn wildcard() -> Self {
        Self::from(vec!["*".to_string()])
    }

    pub fn from(fields: Vec<String>) -> Self {
        Fields {
            list: fields,
            marker: PhantomData
        }
    }
       
} 