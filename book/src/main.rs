#[derive(toql::prelude::Toql)]
struct User {
    #[toql(key)]
    id: u64, 
    name : String
}
#[derive(toql::prelude::Toql)]
struct Book {
    #[toql(key)]
    id: u64, 
    title : String,
    category : String
}

 enum BookCategory {
        Novel,
        Cartoon
    }

struct AuthorAuth {
    author_id: u64
}
struct UserAuth {
    user_id: u64
}

fn api_query1() {

    use toql::prelude::query;

    let user_id = 5;
    let q = query!(User, "*, id eq ?",  user_id);
}

fn api_query2() {
    use toql::prelude::query;

    let q1 = query!(User, "id eq ?", 5);
    let q = query!(User, "*, {}", q1);
}

fn api_query3() {
    use toql::prelude::{query, ToQuery};

    let k = UserKey::from(5);
    let q1 = query!(User, "id eq ?", &k);
    let q2 = query!(User, "*, {}", k.to_query());
}

fn api_query4() {

    use toql::prelude::{query, Query};

    let ks = vec![UserKey::from(1), UserKey::from(2)];

    let qk = ks.iter().collect::<Query<_>>();
    let q4 = query!(User, "*, {}", qk);
}

fn api_query5() {
    use toql::prelude::{query, MapKey, Query};

    let es = vec![User{id:1, name: "Susanne".to_string()}, User{id:2, name:"Pete".to_string()}];

    let qk = es.iter().map_key().collect::<Query<_>>();
    let q5 = query!(User, "*, {}", qk);

}

fn api_query6() {

    use toql::prelude::{query, Query};
   
    impl Into<Query<Book>> for BookCategory {
         fn into(self) -> Query<Book>{
            query!(Book, "category eq ?", 
            match self {
                Novel => "NOVEL",
                Cartoon => "CARTOON"    
            })
        }
    }

    let q = query!(Book, "*, {}", BookCategory::Novel);

}

fn api_query7() {
    use toql::prelude::{Query, QueryFields};

    impl<T> Into<Query<T>> for UserAuth {
        fn into(self) -> Query<T> {
             Query::from(Field::from("authorId").eq(self.user_id))
        }
    }


}
fn api_query8() {
    use toql::prelude::{Query, Field};

    impl<T> Into<Query<T>> for UserAuth {
        fn into(self) -> Query<T> {
             Query::from(Field::from("authorId").eq(self.user_id))
        }
    }
}

fn main() {
    println!("Hello, world!");
}
