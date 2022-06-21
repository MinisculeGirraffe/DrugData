use entity::accounting_entry::*;
use sea_orm_migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220619_234623_create_accounting_table"
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
                    .col(ColumnDef::new(Column::ScheduleId).uuid().not_null())
                    .col(ColumnDef::new(Column::Amount).integer().not_null())
                    .col(ColumnDef::new(Column::Timestamp).timestamp_with_time_zone().not_null())
                    .primary_key(Index::create().col(Column::Id))
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(Entity)
                            .from_col(Column::ScheduleId)
                            .to_tbl(entity::schedule::Entity)
                            .to_col(entity::schedule::Column::Id),
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
