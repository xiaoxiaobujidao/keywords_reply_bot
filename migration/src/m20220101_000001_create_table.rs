use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupReply::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GroupReply::GroupId).big_integer().not_null())
                    .col(ColumnDef::new(GroupReply::Keywords).text().not_null())
                    .col(ColumnDef::new(GroupReply::Reply).text().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupReply::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum GroupReply {
    Table,
    GroupId,
    Keywords,
    Reply,
}
