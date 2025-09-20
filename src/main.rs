use keywords_reply_bot::{Config, DatabaseManager, BotManager};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置
    let config = Config::new()?;
    println!("成功读取配置，开始启动机器人...");
    
    // 验证配置
    config.validate()?;
    
    // 初始化数据库
    let db_manager = DatabaseManager::new(&config.database_url).await?;
    
    // 创建机器人管理器
    let bot_manager = BotManager::new(&config.bot_token, db_manager.connection);
    
    // 验证 bot token
    bot_manager.validate_token().await?;
    
    println!("数据库设置完成，开始监听消息...");
    
    // 开始监听消息
    bot_manager.start_listening().await?;
    
    Ok(())
}
