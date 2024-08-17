use ::entity::{user, user::Entity as User};
use prelude::Uuid;
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_user_by_id(db: &DbConn, id: String) -> Result<Option<user::Model>, DbErr> {
        let uuid = Uuid::parse_str(&id).map_err(|_| DbErr::Custom("Invalid UUID.".to_owned()))?;

        User::find_by_id(uuid).one(db).await
    }

    pub async fn find_user_by_username(
        db: &DbConn,
        username: String,
    ) -> Result<Option<user::Model>, DbErr> {
        User::find()
            .filter(user::Column::Username.eq(username))
            .one(db)
            .await
    }

    pub async fn find_user_by_email(
        db: &DbConn,
        email: String,
    ) -> Result<Option<user::Model>, DbErr> {
        User::find()
            .filter(user::Column::Email.eq(email))
            .one(db)
            .await
    }

    pub async fn find_all_users(db: &DbConn) -> Result<Vec<user::Model>, DbErr> {
        User::find().all(db).await
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
    async fn test_find_user_by_id() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[create_user_with_id("00000000-0000-0000-0000-000000000000")]])
            .into_connection();

        {
            let id = "00000000-0000-0000-0000-000000000000";
            let user = Query::find_user_by_id(&db, id.to_string())
                .await
                .expect("Failed to find user")
                .expect("User not found");

            assert_eq!(user.id, Uuid::parse_str(id).unwrap());
        }

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT "user"."id", "user"."email", "user"."username", "user"."password", CAST("user"."roles" AS text[]), "user"."status" FROM "user" WHERE "user"."id" = $1 LIMIT $2"#,
                [
                    Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                        .unwrap()
                        .into(),
                    1u64.into()
                ]
            )]
        )
    }

    #[tokio::test]
    async fn test_find_user_by_username() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[create_user_with_id("00000000-0000-0000-0000-000000000000")]])
            .into_connection();

        {
            let username = "Test";
            let user = Query::find_user_by_username(&db, username.to_string())
                .await
                .expect("Failed to find user")
                .expect("User not found");

            assert_eq!(user.email, "test@example.com");
        }

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT "user"."id", "user"."email", "user"."username", "user"."password", CAST("user"."roles" AS text[]), "user"."status" FROM "user" WHERE "user"."username" = $1 LIMIT $2"#,
                ["Test".into(), 1u64.into()]
            )]
        )
    }

    #[tokio::test]
    async fn test_find_user_by_email() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[create_user_with_id("00000000-0000-0000-0000-000000000000")]])
            .into_connection();

        {
            let email = "test@example.com";
            let user = Query::find_user_by_email(&db, email.to_string())
                .await
                .expect("Failed to find user")
                .expect("User not found");

            assert_eq!(user.email, "test@example.com");
        }

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT "user"."id", "user"."email", "user"."username", "user"."password", CAST("user"."roles" AS text[]), "user"."status" FROM "user" WHERE "user"."email" = $1 LIMIT $2"#,
                ["test@example.com".into(), 1u64.into()]
            )]
        )
    }

    #[tokio::test]
    async fn test_find_all_users() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[
                create_user_with_id("00000000-0000-0000-0000-000000000000"),
                create_user_with_id("00000000-0000-0000-0000-000000000000"),
            ]])
            .into_connection();

        {
            let users = Query::find_all_users(&db)
                .await
                .expect("Failed to find users");

            assert_eq!(users.len(), 2);
        }

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT "user"."id", "user"."email", "user"."username", "user"."password", CAST("user"."roles" AS text[]), "user"."status" FROM "user""#,
                []
            )]
        )
    }
}
