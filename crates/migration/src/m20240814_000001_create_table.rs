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
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(string(User::Email).unique_key())
                    .col(string(User::Username).unique_key())
                    .col(string(User::Password))
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
                        .default(Expr::value(r#"{user}"#)),
                    )
                    .col(integer(User::Status).default(Expr::value(0)))
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
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(string(Worker::ExternalPath).default("/"))
                    .col(string(Worker::HostName).default("localhost"))
                    .col(string(Worker::NodeName).default("default"))
                    .col(integer(Worker::Port))
                    .col(string(Worker::Entry).default("entry.js"))
                    .col(string(Worker::Code))
                    .col(string(Worker::Name))
                    .col(string_null(Worker::TunnelId))
                    .col(string_null(Worker::Template))
                    .col(uuid(Worker::UserId))
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
