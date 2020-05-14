
pub struct FieldPath<'a> (&'a str);


impl<'a> FieldPath<'a> {

    pub fn from( path: &'a str) -> Self {
        FieldPath(path)
    }
    pub fn ancestors(&self) -> Ancestor {
        Ancestor{pos: self.0.len(), path: self.0}
    }

    pub fn as_str(&self) -> &str {
        self.0
    }
    
  /*   pub fn descendents(&self) -> Decendent {
        Decendent(self)
    } */
}


pub struct Ancestor<'a> {
    pos: usize,
    path: &'a str
}

impl<'a> Iterator for Ancestor<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str>{

       let p =  self.path[0..self.pos].rfind('_');
       match p {
           Some(i) => Some(&self.path[0..i-1]),
           None => None
       }
    }
}



