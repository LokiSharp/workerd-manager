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
