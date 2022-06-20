use entity::session::*;
use sea_orm_migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220619_174222_create_session_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Entity)
                    .col(ColumnDef::new(Column::Id).uuid().not_null())
                    .col(ColumnDef::new(Column::UserId).uuid().not_null())
                    .primary_key(Index::create().col(Column::Id))
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(Entity)
                            .from_col(Column::UserId)
                            .to_tbl(entity::User::Entity)
                            .to_col(entity::User::Column::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
