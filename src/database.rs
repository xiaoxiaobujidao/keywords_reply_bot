use sea_orm::{Database, DatabaseConnection};
use migration::{Migrator, MigratorTrait};
use anyhow::Result;

pub struct DatabaseManager {
    pub connection: DatabaseConnection,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        println!("Connecting to database: {}", database_url);
        
        // 连接数据库
        let connection = Database::connect(database_url).await?;
        
        // 运行迁移
        println!("Running migrations...");
        Migrator::up(&connection, None).await?;
        println!("Migrations completed successfully!");
        
        Ok(DatabaseManager { connection })
    }
}
