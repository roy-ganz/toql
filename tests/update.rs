use fixtures::tree1::Alpha;

use toql::prelude::fields;
 use toql::backend::Backend;

mod fixtures;
mod mock;

#[tokio::test]
async fn update_tree1() {

       let mut mock = mock::Mock::default();
       let mut entities= vec![fixtures::tree1::Alpha::default()];

        // Update foreign key for Beta in Alpha (no join required)
        mock.update(&mut entities, fields!(Alpha, "beta")).await.unwrap();

        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
        }
          
        // Update all fields on join beta
        mock.clear();
        mock.update(&mut entities, fields!(Alpha, "beta_*")).await.unwrap();

        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
           
        }
       
        // Update all fields on merge gamma
        mock.clear();
        mock.update(&mut entities, fields!(Alpha, "gamma_*")).await.unwrap();
        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
        }
       // Replace vec gamma (delete + insert)
        mock.clear();
        mock.update(&mut entities, fields!(Alpha, "gamma")).await.unwrap();
        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
        }
       

}




/* 


#[tokio::test]
async fn insert_tree1() {

       use toql::prelude::paths;

      
       let mut mock = mock::Mock::default();

       let mut entities= vec![fixtures::tree1::Alpha::default()];


        // Insert top entity
        mock.insert(&mut entities, paths!(top)).await.unwrap();

        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
        }
          
        // Insert joined beta and top entity (in correct order)
        mock.clear();
        mock.insert(&mut entities, paths!(Alpha, "beta")).await.unwrap();

        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
           
        }
       
        // Insert merge gamma and top entity (in correct order)
        mock.clear();
        mock.insert(&mut entities, paths!(Alpha, "gamma")).await.unwrap();
        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
        }
       
       // Insert merge gamma, join beta and top entity (in correct order)
        mock.clear();
        mock.insert(&mut entities, paths!(Alpha, "beta, gamma")).await.unwrap();
        for (e, s) in mock.sqls.iter().enumerate(){
               println!("[{}] {}", e, s.to_unsafe_string());
        }
       

} */