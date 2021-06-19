use fixtures::tree1::Alpha;
use toql::backend::command::update_command::UpdateCommand;
use toql::prelude::{Result, Sql, SqlMapperRegistry, fields};


mod fixtures;



#[test]
fn update_tree1() {

        let registry = SqlMapperRegistry::new();
        let mut entities = vec![Alpha::default()];
        let fields= fields!(Alpha, "beta_*, gamma_*");
        let mut sqls : Vec<Sql>;
        
        let mut collect_sql = |sql: Sql| -> Result<()> {
            // collect statements
             dbg!(sql.to_unsafe_string());
             sqls.push(sql);
             Ok(())
        };

        let cmd = UpdateCommand::new(&registry, &mut collect_sql);
          
        cmd.run(&mut entities, fields!(Alpha, "beta_*, gamma_*")).unwrap();
        // assert statements

        sqls.clear();
        cmd.run(&mut entities, fields!(Alpha, "*,beta_*, gamma_*")).unwrap();
         // assert statements

}