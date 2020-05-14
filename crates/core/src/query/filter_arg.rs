
/// A trait to convert a simple datatype into a filter argument. Used by builder functions. Not very interesting ;)
pub trait FilterArg {
    fn to_sql(&self) -> String;
}

impl FilterArg for &str {
    fn to_sql(&self) -> String {
         self.to_string()
        /* let mut s = String::from("'");
        // TODO escape for sql
        s.push_str(*self);
        s.push('\'');
        s */
    }
}
// TODO combine with above
impl FilterArg for String {
    fn to_sql(&self) -> String {
         self.to_string()
      /*   let mut s = String::from("'");
        // TODO escape for sql
        s.push_str(self);
        s.push('\'');
        s */
    }
}
impl FilterArg for &String {
    fn to_sql(&self) -> String {
        self.to_string()
       /*  let mut s = String::from("'");
        // TODO escape for sql
        s.push_str(self.as_str());
        s.push('\'');
        s */
    }
}

macro_rules! impl_num_filter_arg {
    ($($mty:ty),+) => {
        $(
            impl FilterArg for $mty {
                 fn to_sql(&self) -> String {
                    self.to_string()
                 }
            }
            impl<'a> FilterArg for &'a $mty {
                 fn to_sql(&self) -> String {
                    self.to_string()
                 }
            }
        )+
    }
}

impl_num_filter_arg!(usize, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

 impl<U, T: Into<Query<U>>> Into<Query<U>> for Vec<T> {
    fn into(self) -> Query<U> {
        let mut query = Query::<U>::new();
        for key in self {
            query = query.or(key);
        }
        query
    }
}

impl<U,T: Into<Query<U>> + Clone> Into<Query<U>> for &Vec<T> {
    fn into(self) -> Query<U> {
        let mut query = Query::<U>::new();
        for key in self {
            query = query.or(key.clone());
        }
        query
    }
} 
 
impl<U,T: Into<Query<U>> + Clone> Into<Query<U>> for &[T] {
    fn into(self) -> Query<U> {
        let mut query = Query::<U>::new();
        for key in self {
            query = query.or(key.clone());
        }
        query
    }
}   

impl FilterArg for bool {
    fn to_sql(&self) -> String {
        String::from(if *self == true { "1" } else { "0" })
    }
}

impl FilterArg for &bool {
    fn to_sql(&self) -> String {
        String::from(if **self == true { "1" } else { "0" })
    }
}
