use fixtures::tree1::Alpha;

use toql::prelude::fields;
 use toql::backend::Backend;

mod fixtures;
mod mock;

#[test]
fn update_tree1() {

        let mut mock = mock::Mock::default();

       let mut entities= vec![fixtures::tree1::Alpha::default()];


        // Same as beta_*
        mock.update(&mut entities, fields!(Alpha, "beta")).unwrap();

        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
        }

          
        // Update all fields on join beta
        mock.clear();
        mock.update(&mut entities, fields!(Alpha, "beta_*")).unwrap();

        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
           
        }
       
        // Update all fields on merge gamma
        mock.clear();
        mock.update(&mut entities, fields!(Alpha, "gamma_*")).unwrap();
        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
        }
       // Replace vec gamma (delete + insert)
        mock.clear();
        mock.update(&mut entities, fields!(Alpha, "gamma")).unwrap();
        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
        }
       

}







#[test]
fn insert_tree1() {

       use toql::prelude::paths;

      
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