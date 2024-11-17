use sqlx::{Error, MySqlPool};

pub async fn database_connetion() -> Result<MySqlPool, Error> {
    MySqlPool::connect("mysql://root:password@localhost:3306/brainly").await
}
