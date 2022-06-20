use entity::schedule::*;
use sea_orm_migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220619_230031_create_schedule_table"
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
                    .col(ColumnDef::new(Column::AddedAt).date_time().not_null())
                    .col(ColumnDef::new(Column::Cron).string().not_null())
                    .col(ColumnDef::new(Column::DrugName).string().not_null())
                    .col(
                        ColumnDef::new(Column::PillCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Column::PillAmount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .primary_key(Index::create().col(Column::Id))
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(Entity)
                            .from_col(Column::UserId)
                            .to_tbl(entity::user::Entity)
                            .to_col(entity::user::Column::Id),
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
