use std::env;
use std::path::Path;
use std::fs;
use std::io::{self, Write};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub bot_token: String,
    pub database_url: String,
}

impl Config {
    pub fn new() -> Result<Self> {
        let token_file = "bot_token";
        let bot_token = if !Path::new(token_file).exists() {
            // 文件不存在，等待用户输入
            println!("未找到 bot_token 文件");
            println!("请输入你的 Telegram Bot Token:");
            print!("Token: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let token = input.trim().to_string();
            
            if token.is_empty() {
                return Err(anyhow::anyhow!("Bot token 不能为空"));
            }
            
            // 询问是否保存到文件
            println!("是否将 token 保存到 {} 文件中？(y/n): ", token_file);
            print!("选择: ");
            io::stdout().flush()?;
            
            let mut save_choice = String::new();
            io::stdin().read_line(&mut save_choice)?;
            
            if save_choice.trim().to_lowercase() == "y" || save_choice.trim().to_lowercase() == "yes" {
                fs::write(token_file, &token)?;
                println!("Token 已保存到 {} 文件", token_file);
            }
            
            token
        } else {
            // 文件存在，读取内容
            let token = fs::read_to_string(token_file)
                .map_err(|e| anyhow::anyhow!("无法读取 bot_token 文件: {}", e))?;
            let token = token.trim().to_string();
            
            if token.is_empty() {
                eprintln!("错误: bot_token 文件为空！");
                eprintln!("请输入你的 Telegram Bot Token:");
                print!("Token: ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let new_token = input.trim().to_string();
                
                if new_token.is_empty() {
                    return Err(anyhow::anyhow!("Bot token 不能为空"));
                }
                
                // 更新文件内容
                fs::write(token_file, &new_token)?;
                new_token
            } else {
                token
            }
        };
        
        // 从环境变量或使用默认的 SQLite 数据库路径
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:keywords_reply_bot.db".to_string());
        
        Ok(Config {
            bot_token,
            database_url,
        })
    }
    
    pub fn validate(&self) -> Result<()> {
        // 检查数据库文件是否存在，如果不存在则创建
        if self.database_url.starts_with("sqlite:") {
            let db_path = self.database_url.strip_prefix("sqlite:").unwrap();
            if !Path::new(db_path).exists() {
                println!("Database file not found, creating: {}", db_path);
                // 创建空的数据库文件
                std::fs::File::create(db_path)?;
                println!("Database file created successfully!");
            }
        }
        
        Ok(())
    }
}
