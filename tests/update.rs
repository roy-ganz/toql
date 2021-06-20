use fixtures::tree1::Alpha;
use ops::update::TestUpdate;
use toql::prelude::fields;
 use toql::backend::ops::update::Update;

mod fixtures;
pub mod ops;


#[test]
fn update_tree1() {

       let mut ops = TestUpdate::default();

       let entities= vec![fixtures::tree1::Alpha::default()];

        let fields= fields!(Alpha, "beta_*, gamma_*");
          
        ops.update(&mut entities, fields!(Alpha, "beta_*, gamma_*")).unwrap();
        // assert statements

        ops.sqls.clear();
        ops.update(&mut entities, fields!(Alpha, "*,beta_*, gamma_*")).unwrap();
         // assert statements

}