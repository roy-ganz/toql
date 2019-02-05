use crate::buildable::Buildable;
use crate::sql_mapper::SqlMapper;


pub struct UserDto {
    pub id: u32,
}

impl Buildable<UserDto> for UserDto {
    fn build(row: &str) -> UserDto {
        UserDto { id: 2 }
    }
}

impl UserDto {
    pub fn find_for_toql(mapper: &SqlMapper, toql: &str, offset: u32, limit: u32) -> UserDto {
        // mapping order is select order
        // unselected fields are null
        // let r: MapperResult = mapper.build(toql);
        // let r: MapperResult = mapper.build(toql, "examinee");
        // println!("Select {} FROM User u WHERE {} ORDER BY {}",
        // mapper.select, mapper.where, mapper.order_by)
        // println!("SELECT ROW COUNT");
       
        //let s  = mapper.build_sql("toql");
        UserDto { id: 6 }
    }
    pub fn find_for_id(id: u32) -> UserDto {
        UserDto { id: 5 }
    }
    pub fn find_for_ids(id: Vec<u32>) -> Vec<UserDto> {
        vec![UserDto { id: 5 }]
    }

    pub fn delete(users: Vec<UserDto>) {
        println!("Deleting {:?}", users.into_iter().map(|u| u.id).collect::<Vec<_>>());
    }

    pub fn update(users: Vec<UserDto>) {}
}
