//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.7

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "site_urls")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub site_id: i32,
    pub url: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::sites::Entity",
        from = "Column::SiteId",
        to = "super::sites::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Sites,
}

impl Related<super::sites::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sites.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
