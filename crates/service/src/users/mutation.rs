use ::entity::{user, user::Entity as User};
use prelude::Uuid;
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create_user(
        db: &DbConn,
        email: String,
        username: String,
        password: String,
    ) -> Result<user::ActiveModel, DbErr> {
        user::ActiveModel {
            email: Set(email),
            password: Set(password),
            username: Set(username),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_user(
        db: &DbConn,
        id: String,
        email: String,
        username: String,
        password: String,
    ) -> Result<user::Model, DbErr> {
        let uuid = Uuid::parse_str(&id).map_err(|_| DbErr::Custom("Invalid UUID.".to_owned()))?;

        let user: user::ActiveModel = User::find_by_id(uuid)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find user.".to_owned()))
            .map(Into::into)?;

        user::ActiveModel {
            id: user.id,
            email: Set(email),
            username: Set(username),
            password: Set(password),
            ..user
        }
        .update(db)
        .await
    }

    pub async fn delete_user(db: &DbConn, id: String) -> Result<DeleteResult, DbErr> {
        let uuid = Uuid::parse_str(&id).map_err(|_| DbErr::Custom("Invalid UUID.".to_owned()))?;

        let user: user::ActiveModel = User::find_by_id(uuid)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find user.".to_owned()))
            .map(Into::into)?;

        user.delete(db).await
    }
}

#[cfg(test)]
mod tests {
    use ::entity::sea_orm_active_enums::RoleEnum;

    use super::*;

    fn create_user_with_id(id: &str) -> user::Model {
        user::Model {
            id: Uuid::parse_str(id).unwrap(),
            email: "test@example.com".to_owned(),
            username: "Test".to_owned(),
            password: "password".to_owned(),
            roles: vec![RoleEnum::User],
            status: 0,
        }
    }

    #[tokio::test]
    async fn test_create_user() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[create_user_with_id("00000000-0000-0000-0000-000000000000")]])
            .into_connection();

        {
            let user = Mutation::create_user(
                &db,
                "test@example.com".to_owned(),
                "Test".to_owned(),
                "password".to_owned(),
            )
            .await
            .expect("Failed to create user");

            assert_eq!(
                user,
                user::ActiveModel {
                    id: Unchanged(Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()),
                    email: Unchanged("test@example.com".to_string()),
                    username: Unchanged("Test".to_string()),
                    password: Unchanged("password".to_string()),
                    roles: Unchanged(vec![RoleEnum::User]),
                    status: Unchanged(0),
                }
            );
        }

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"INSERT INTO "user" ("email", "username", "password") VALUES ($1, $2, $3) RETURNING "id", "email", "username", "password", CAST("roles" AS text[]), "status""#,
                ["test@example.com".into(), "Test".into(), "password".into()]
            )]
        )
    }

    #[tokio::test]
    async fn test_update_user() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                [create_user_with_id("00000000-0000-0000-0000-000000000000")],
                [create_user_with_id("00000000-0000-0000-0000-000000000000")],
            ])
            .into_connection();

        {
            let user = Mutation::update_user(
                &db,
                "00000000-0000-0000-0000-000000000000".to_owned(),
                "test@example.com".to_owned(),
                "Test".to_owned(),
                "password".to_owned(),
            )
            .await
            .expect("Failed to update user");

            assert_eq!(
                user,
                user::Model {
                    id: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
                    email: "test@example.com".to_string(),
                    username: "Test".to_string(),
                    password: "password".to_string(),
                    roles: vec![RoleEnum::User],
                    status: 0,
                }
            );
        }

        assert_eq!(
            db.into_transaction_log(),
            [
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"SELECT "user"."id", "user"."email", "user"."username", "user"."password", CAST("user"."roles" AS text[]), "user"."status" FROM "user" WHERE "user"."id" = $1 LIMIT $2"#,
                    [
                        Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                            .unwrap()
                            .into(),
                        1u64.into()
                    ]
                ),
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"UPDATE "user" SET "email" = $1, "username" = $2, "password" = $3 WHERE "user"."id" = $4 RETURNING "id", "email", "username", "password", CAST("roles" AS text[]), "status""#,
                    [
                        "test@example.com".into(),
                        "Test".into(),
                        "password".into(),
                        Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                            .unwrap()
                            .into(),
                    ]
                )
            ]
        )
    }

    #[tokio::test]
    async fn test_delete_user() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[create_user_with_id("00000000-0000-0000-0000-000000000000")]])
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();

        {
            let result =
                Mutation::delete_user(&db, "00000000-0000-0000-0000-000000000000".to_owned())
                    .await
                    .expect("Failed to delete user");

            assert_eq!(result.rows_affected, 1);
        }

        assert_eq!(
            db.into_transaction_log(),
            [
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"SELECT "user"."id", "user"."email", "user"."username", "user"."password", CAST("user"."roles" AS text[]), "user"."status" FROM "user" WHERE "user"."id" = $1 LIMIT $2"#,
                    [
                        Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                            .unwrap()
                            .into(),
                        1u64.into()
                    ]
                ),
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"DELETE FROM "user" WHERE "user"."id" = $1"#,
                    [Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                        .unwrap()
                        .into()]
                )
            ]
        )
    }
}
