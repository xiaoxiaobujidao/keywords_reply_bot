use frankenstein::client_reqwest::Bot;
use frankenstein::methods::GetUpdatesParams;
use frankenstein::updates::UpdateContent;
use frankenstein::AsyncTelegramApi;
use sea_orm::DatabaseConnection;
use anyhow::Result;
use crate::handlers::MessageHandler;

pub struct BotManager {
    api: Bot,
    db: DatabaseConnection,
    message_handler: MessageHandler,
}

impl BotManager {
    pub fn new(bot_token: &str, db: DatabaseConnection) -> Self {
        let api = Bot::new(bot_token);
        let message_handler = MessageHandler::new(db.clone());
        
        BotManager {
            api,
            db,
            message_handler,
        }
    }
    
    pub fn get_database(&self) -> &DatabaseConnection {
        &self.db
    }
    
    pub async fn validate_token(&self) -> Result<()> {
        match self.api.get_me().await {
            Ok(response) => {
                let user = response.result;
                println!("机器人信息: @{} ({})", 
                    user.username.unwrap_or_default(), 
                    user.first_name
                );
                Ok(())
            }
            Err(e) => {
                eprintln!("错误: 无法连接到 Telegram API: {}", e);
                eprintln!("请检查 bot_token 是否正确");
                Err(e.into())
            }
        }
    }
    
    pub async fn start_listening(&self) -> Result<()> {
        let mut update_params = GetUpdatesParams::builder().build();
        
        loop {
            match self.api.get_updates(&update_params).await {
                Ok(response) => {
                    for update in response.result {
                        if let UpdateContent::Message(message) = update.content {
                            let api_clone = self.api.clone();
                            let handler = self.message_handler.clone();
                            
                            tokio::spawn(async move {
                                if let Err(e) = handler.handle_message(api_clone, *message).await {
                                    eprintln!("处理消息时出错: {}", e);
                                }
                            });
                        }
                        update_params.offset = Some(i64::from(update.update_id) + 1);
                    }
                }
                Err(e) => {
                    eprintln!("获取更新时出错: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
}
