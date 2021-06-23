use fixtures::tree1::Alpha;
use toql::prelude::paths;
use toql::backend::ops::insert::Insert;


mod fixtures;
mod mock;


#[test]
fn insert_tree1() {

       let mut mock = mock::Mock::default();
       let mut entities= vec![fixtures::tree1::Alpha::default()];

        // Insert top entity
        mock.insert(&mut entities, paths!(top)).unwrap();

        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
        }
          
        // Insert join beta and top entity (in correct order)
        mock.clear();
        mock.insert(&mut entities, paths!(Alpha, "beta")).unwrap();

        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
           
        }
       
        // Insert merge gamma and top entity (in correct order)
        mock.clear();
        mock.insert(&mut entities, paths!(Alpha, "gamma")).unwrap();
        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
        }
       
       // Insert merge gamma, join beta and top entity (in correct order)
        mock.clear();
        mock.insert(&mut entities, paths!(Alpha, "beta, gamma")).unwrap();
        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
        }
       

}