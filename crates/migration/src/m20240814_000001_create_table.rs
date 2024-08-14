use extension::postgres::Type;
use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(RoleEnum)
                    .values(RoleVariants::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        uuid(User::Id)
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(string(User::Email).not_null().unique_key())
                    .col(string(User::Username).not_null().unique_key())
                    .col(string(User::Password).not_null())
                    .col(
                        array(
                            User::Roles,
                            ColumnType::Enum {
                                name: SeaRc::new(RoleEnum),
                                variants: RoleVariants::iter()
                                    .map(|v| SeaRc::new(v))
                                    .collect::<Vec<_>>(),
                            },
                        )
                        .not_null()
                        .default(Expr::value(r#"{user}"#)),
                    )
                    .col(integer(User::Status).not_null().default(Expr::value(0)))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Worker::Table)
                    .if_not_exists()
                    .col(
                        uuid(Worker::Id)
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(string(Worker::ExternalPath).not_null().default("/"))
                    .col(string(Worker::HostName).not_null().default("localhost"))
                    .col(string(Worker::NodeName).not_null().default("default"))
                    .col(integer(Worker::Port).not_null())
                    .col(string(Worker::Entry).not_null().default("entry.js"))
                    .col(string(Worker::Code).not_null())
                    .col(string(Worker::Name).not_null())
                    .col(string(Worker::TunnelId))
                    .col(string(Worker::Template))
                    .col(uuid(Worker::UserId).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("worker_user_id_fkey")
                    .from(Worker::Table, Worker::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("worker_user_id_fkey")
                    .table(Worker::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Worker::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(RoleEnum).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Email,
    Username,
    Password,
    Roles,
    Status,
}

#[derive(DeriveIden)]
struct RoleEnum;

#[derive(DeriveIden, EnumIter)]
enum RoleVariants {
    User,
    Admin,
}

#[derive(DeriveIden)]
enum Worker {
    Table,
    Id,
    ExternalPath,
    HostName,
    NodeName,
    Port,
    Entry,
    Code,
    Name,
    TunnelId,
    Template,
    UserId,
}
