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
            .filter(user::Column::Username.contains(username))
            .one(db)
            .await
    }

    pub async fn find_user_by_email(
        db: &DbConn,
        email: String,
    ) -> Result<Option<user::Model>, DbErr> {
        User::find()
            .filter(user::Column::Email.contains(email))
            .one(db)
            .await
    }
}
