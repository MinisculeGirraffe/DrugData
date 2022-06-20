pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_user_table;
mod m20220618_162459_create_product_table;
mod m20220619_174222_create_session_table;




pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_user_table::Migration),
            Box::new(m20220618_162459_create_product_table::Migration),
            Box::new(m20220619_174222_create_session_table::Migration),
        ]
    }
}


