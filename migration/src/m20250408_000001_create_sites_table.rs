use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Sites::Table)
                    .if_not_exists()
                    .col(pk_auto(Sites::Id))
                    .col(string(Sites::Domain).not_null())
                    .col(
                        ColumnDef::new(Sites::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Sites::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Sites {
    Table,
    Id,
    Domain,
    CreatedAt,
}
