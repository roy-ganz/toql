



// Implement for fields ands paths
pub trait UpdateField {

    fn into_field( self) -> String;

}



impl UpdateField for crate::query::field::Field  {
    fn into_field(self) -> String {
      self.name
    }
}



impl UpdateField for crate::query::wildcard::Wildcard  {
    fn into_field(self) ->String {
      self.path
    }
}
