use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub struct PgClient {
    pool: PgPool,
}

impl PgClient {
    pub async fn new(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        database: &str,
    ) -> crate::error::Result<Self> {
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, database
        );
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;
        
        Ok(Self { pool })
    }
    
    pub async fn get_databases(&self) -> crate::error::Result<Vec<String>> {
        let db_names = sqlx::query_scalar(
            "SELECT datname FROM pg_database WHERE datallowconn = true ORDER BY datname",
        )
            .fetch_all(&self.pool)
            .await?;
        
        Ok(db_names)
    }
}