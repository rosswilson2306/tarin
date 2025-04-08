use sea_orm_migration::{prelude::*, schema::*};

use super::m20250408_000001_create_sites_table::Sites;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SiteUrls::Table)
                    .if_not_exists()
                    .col(pk_auto(SiteUrls::Id))
                    .col(ColumnDef::new(SiteUrls::SiteId).integer().not_null())
                    .col(string(SiteUrls::Url).not_null())
                    .col(
                        ColumnDef::new(SiteUrls::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-site-url-site_id")
                            .from(SiteUrls::Table, SiteUrls::SiteId)
                            .to(Sites::Table, Sites::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SiteUrls::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SiteUrls {
    Table,
    Id,
    SiteId,
    Url,
    CreatedAt,
}
