use ::entity::{worker, worker::Entity as Worker};
use prelude::Uuid;
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create_worker(
        db: &DbConn,
        name: String,
        port: i32,
        code: String,
        user_id: String,
    ) -> Result<worker::ActiveModel, DbErr> {
        worker::ActiveModel {
            name: Set(name),
            port: Set(port),
            code: Set(code),
            user_id: Set(
                Uuid::parse_str(&user_id).map_err(|_| DbErr::Custom("Invalid UUID.".to_owned()))?
            ),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_worker(
        db: &DbConn,
        id: String,
        external_path: String,
        host_name: String,
        node_name: String,
        port: i32,
        code: String,
        name: String,
        tunnel_id: Option<String>,
        template: Option<String>,
    ) -> Result<worker::Model, DbErr> {
        let uuid = Uuid::parse_str(&id).map_err(|_| DbErr::Custom("Invalid UUID.".to_owned()))?;

        let worker: worker::ActiveModel = Worker::find_by_id(uuid)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find worker.".to_owned()))
            .map(Into::into)?;

        worker::ActiveModel {
            id: worker.id,
            external_path: Set(external_path),
            host_name: Set(host_name),
            node_name: Set(node_name),
            port: Set(port),
            code: Set(code),
            name: Set(name),
            tunnel_id: Set(tunnel_id),
            template: Set(template),
            ..worker
        }
        .update(db)
        .await
    }

    pub async fn delete_worker(db: &DbConn, id: String) -> Result<DeleteResult, DbErr> {
        let uuid = Uuid::parse_str(&id).map_err(|_| DbErr::Custom("Invalid UUID.".to_owned()))?;

        let worker: worker::ActiveModel = Worker::find_by_id(uuid)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find worker.".to_owned()))
            .map(Into::into)?;

        worker.delete(db).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_worker_with_id(id: &str) -> worker::Model {
        worker::Model {
            id: Uuid::parse_str(id).unwrap(),
            external_path: "/".to_string(),
            host_name: "localhost".to_string(),
            node_name: "default".to_string(),
            port: 80,
            entry: "entry.js".to_string(),
            code: "".to_string(),
            name: "Test".to_string(),
            tunnel_id: None,
            template: None,
            user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
        }
    }

    #[tokio::test]
    async fn test_create_worker() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[create_worker_with_id(
                "00000000-0000-0000-0000-000000000000",
            )]])
            .into_connection();

        {
            let worker = Mutation::create_worker(
                &db,
                "Test".to_string(),
                80,
                "".to_string(),
                "00000000-0000-0000-0000-000000000000".to_string(),
            )
            .await
            .expect("Failed to create user");

            assert_eq!(
                worker,
                worker::ActiveModel {
                    id: Unchanged(Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()),
                    external_path: Unchanged("/".to_string()),
                    host_name: Unchanged("localhost".to_string()),
                    node_name: Unchanged("default".to_string()),
                    port: Unchanged(80),
                    entry: Unchanged("entry.js".to_string()),
                    code: Unchanged("".to_string()),
                    name: Unchanged("Test".to_string()),
                    tunnel_id: Unchanged(None),
                    template: Unchanged(None),
                    user_id: Unchanged(
                        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
                    ),
                }
            );
        }

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"INSERT INTO "worker" ("port", "code", "name", "user_id") VALUES ($1, $2, $3, $4) RETURNING "id", "external_path", "host_name", "node_name", "port", "entry", "code", "name", "tunnel_id", "template", "user_id""#,
                [
                    80.into(),
                    "".into(),
                    "Test".into(),
                    Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                        .unwrap()
                        .into()
                ]
            )]
        )
    }

    #[tokio::test]
    async fn test_update_worker() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                [create_worker_with_id(
                    "00000000-0000-0000-0000-000000000000",
                )],
                [create_worker_with_id(
                    "00000000-0000-0000-0000-000000000000",
                )],
            ])
            .into_connection();

        {
            let worker = Mutation::update_worker(
                &db,
                "00000000-0000-0000-0000-000000000000".to_string(),
                "/".to_string(),
                "localhost".to_string(),
                "default".to_string(),
                80,
                "".to_string(),
                "Test".to_string(),
                None,
                None,
            )
            .await
            .expect("Failed to update user");

            assert_eq!(
                worker,
                worker::Model {
                    id: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
                    external_path: "/".to_string(),
                    host_name: "localhost".to_string(),
                    node_name: "default".to_string(),
                    port: 80,
                    entry: "entry.js".to_string(),
                    code: "".to_string(),
                    name: "Test".to_string(),
                    tunnel_id: None,
                    template: None,
                    user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
                }
            );
        }

        assert_eq!(
            db.into_transaction_log(),
            [
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"SELECT "worker"."id", "worker"."external_path", "worker"."host_name", "worker"."node_name", "worker"."port", "worker"."entry", "worker"."code", "worker"."name", "worker"."tunnel_id", "worker"."template", "worker"."user_id" FROM "worker" WHERE "worker"."id" = $1 LIMIT $2"#,
                    [
                        Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                            .unwrap()
                            .into(),
                        1u64.into()
                    ]
                ),
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"UPDATE "worker" SET "external_path" = $1, "host_name" = $2, "node_name" = $3, "port" = $4, "code" = $5, "name" = $6, "tunnel_id" = $7, "template" = $8 WHERE "worker"."id" = $9 RETURNING "id", "external_path", "host_name", "node_name", "port", "entry", "code", "name", "tunnel_id", "template", "user_id""#,
                    [
                        "/".into(),
                        "localhost".into(),
                        "default".into(),
                        80.into(),
                        "".into(),
                        "Test".into(),
                        Option::<String>::None.into(),
                        Option::<String>::None.into(),
                        Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                            .unwrap()
                            .into()
                    ]
                )
            ]
        )
    }

    #[tokio::test]
    async fn test_delete_worker() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                [create_worker_with_id(
                    "00000000-0000-0000-0000-000000000000",
                )],
                [create_worker_with_id(
                    "00000000-0000-0000-0000-000000000000",
                )],
            ])
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();

        {
            let result =
                Mutation::delete_worker(&db, "00000000-0000-0000-0000-000000000000".to_string())
                    .await
                    .expect("Failed to delete user");

            assert_eq!(result.rows_affected, 1);
        }

        assert_eq!(
            db.into_transaction_log(),
            [
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"SELECT "worker"."id", "worker"."external_path", "worker"."host_name", "worker"."node_name", "worker"."port", "worker"."entry", "worker"."code", "worker"."name", "worker"."tunnel_id", "worker"."template", "worker"."user_id" FROM "worker" WHERE "worker"."id" = $1 LIMIT $2"#,
                    [
                        Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                            .unwrap()
                            .into(),
                        1u64.into()
                    ]
                ),
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"DELETE FROM "worker" WHERE "worker"."id" = $1"#,
                    [Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                        .unwrap()
                        .into()]
                )
            ]
        )
    }
}
