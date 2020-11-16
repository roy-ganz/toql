



// Implement for fields ands paths
pub trait UpdateField {

    fn as_field<'a>(&'a self) -> &'a str;

}



impl UpdateField for crate::query::field::Field  {
    fn as_field<'a>(&'a self) -> &'a str {
      self.name.as_str()
    }
}



impl UpdateField for crate::query::wildcard::Wildcard  {
    fn as_field<'a>(&'a self) -> &'a str {
      self.path.as_str()
    }
}
