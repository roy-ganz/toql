/// A Selection tells the deserializer which fields in struct are selected 
 /// and need to be deserialized. The bits are in order of struct fields.
type Selection = Vec<bool>;