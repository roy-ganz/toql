use toql::prelude::Toql;


 #[derive(Debug, PartialEq, Toql)]
pub struct Alpha {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(merge())]
    beta1: Vec<Beta>,  // Preselected merge
        
    #[toql(merge())]
    beta2: Option<Vec<Beta>>,  // Selectable merge
}

#[derive(Debug, PartialEq, Toql)]
pub struct Beta {
    #[toql(key)]
    id: u64,
    text: String,
} 
