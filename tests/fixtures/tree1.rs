use toql::prelude::{Join, Toql};


#[derive(Debug, PartialEq, Toql)]
pub struct Alpha {
    #[toql(key)]
    id: u64,
    text: String,

    #[toql(join())]
    beta: Join<Beta>,

    #[toql(merge())]
    gamma: Vec<Gamma>
}

#[derive(Debug, PartialEq, Toql)]
pub struct Beta {
    #[toql(key)]
    id: u64,
    text: String,
}

#[derive(Debug, PartialEq, Toql)]
pub struct Gamma {
    #[toql(key)]
    id: u64,
    text: String
}

impl Default for Alpha {
    fn default() -> Alpha {
        Alpha {
            id: 1,
            text: "Alpha".to_string(),
            beta: Join::Entity(Box::new(Beta {
                id: 11,
                text: "Beta".to_string(),
            })),
            gamma: vec![
                Gamma {
                    id: 12,
                    text: "Gamma1".to_string(),
                },
                Gamma {
                    id: 13,
                    text: "Gamma2".to_string(),
                },
            ],
        }
    }
}
/*
#[derive(Debug, PartialEq)]
pub struct Alpha {
    id: u64,
    text: String,

    beta: Join<Beta>,

    gamma: Vec<Gamma>,
}

#[derive(Debug, PartialEq)]
pub struct Beta {
    id: u64,
    text: String,
}

#[derive(Debug, PartialEq)]
pub struct Gamma {
    id: u64,
    text: String,
}
*/


