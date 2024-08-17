use ::entity::{worker, worker::Entity as Worker};
use prelude::Uuid;
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_worker_by_id(
        db: &DbConn,
        id: String,
    ) -> Result<Option<worker::Model>, DbErr> {
        let uuid = Uuid::parse_str(&id).map_err(|_| DbErr::Custom("Invalid UUID.".to_owned()))?;

        Worker::find_by_id(uuid).one(db).await
    }

    pub async fn find_all_workers(db: &DbConn) -> Result<Vec<worker::Model>, DbErr> {
        Worker::find().all(db).await
    }

    pub async fn find_user_workers_with_user_id(
        db: &DbConn,
        user_id: String,
    ) -> Result<Vec<worker::Model>, DbErr> {
        Worker::find()
            .filter(worker::Column::UserId.eq(Uuid::parse_str(&user_id).unwrap()))
            .all(db)
            .await
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
            port: 12345,
            entry: "entry.js".to_string(),
            code: "".to_string(),
            name: "Test".to_string(),
            tunnel_id: None,
            template: None,
            user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
        }
    }

    #[tokio::test]
    async fn test_find_worker_by_id() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[create_worker_with_id(
                "00000000-0000-0000-0000-000000000000",
            )]])
            .into_connection();

        {
            let id = "00000000-0000-0000-0000-000000000000";
            let worker = Query::find_worker_by_id(&db, id.to_string())
                .await
                .expect("Failed to find worker")
                .expect("worker not found");

            assert_eq!(worker.id, Uuid::parse_str(id).unwrap());
        }

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT "worker"."id", "worker"."external_path", "worker"."host_name", "worker"."node_name", "worker"."port", "worker"."entry", "worker"."code", "worker"."name", "worker"."tunnel_id", "worker"."template", "worker"."user_id" FROM "worker" WHERE "worker"."id" = $1 LIMIT $2"#,
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
    async fn test_find_all_workers() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[
                create_worker_with_id("00000000-0000-0000-0000-000000000000"),
                create_worker_with_id("00000000-0000-0000-0000-000000000000"),
            ]])
            .into_connection();

        {
            let workers = Query::find_all_workers(&db)
                .await
                .expect("Failed to find workers");

            assert_eq!(workers.len(), 2);
        }

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT "worker"."id", "worker"."external_path", "worker"."host_name", "worker"."node_name", "worker"."port", "worker"."entry", "worker"."code", "worker"."name", "worker"."tunnel_id", "worker"."template", "worker"."user_id" FROM "worker""#,
                []
            )]
        )
    }

    #[tokio::test]
    async fn test_find_user_workers_with_user_id() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[
                create_worker_with_id("00000000-0000-0000-0000-000000000000"),
                create_worker_with_id("00000000-0000-0000-0000-000000000000"),
            ]])
            .into_connection();

        {
            let workers = Query::find_user_workers_with_user_id(
                &db,
                "00000000-0000-0000-0000-000000000000".to_string(),
            )
            .await
            .expect("Failed to find workers");

            assert_eq!(workers.len(), 2);
        }

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT "worker"."id", "worker"."external_path", "worker"."host_name", "worker"."node_name", "worker"."port", "worker"."entry", "worker"."code", "worker"."name", "worker"."tunnel_id", "worker"."template", "worker"."user_id" FROM "worker" WHERE "worker"."user_id" = $1"#,
                [Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                    .unwrap()
                    .into()]
            )]
        )
    }
}
