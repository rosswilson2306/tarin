pub use sea_orm_migration::prelude::*;

mod m20250408_000001_create_sites_table;
mod m20250408_000002_create_site_urls_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250408_000001_create_sites_table::Migration),
            Box::new(m20250408_000002_create_site_urls_table::Migration),
        ]
    }
}
