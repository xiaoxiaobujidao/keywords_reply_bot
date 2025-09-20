use frankenstein::client_reqwest::Bot;
use frankenstein::types::{Message, ChatMember, MessageEntityType};
use frankenstein::methods::{SendMessageParams, GetChatMemberParams};
use frankenstein::ParseMode;
use frankenstein::AsyncTelegramApi;
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait};
use anyhow::Result;
use crate::entities::group_reply::{self, Entity as GroupReplyEntity};

#[derive(Clone)]
pub struct MessageHandler {
    db: DatabaseConnection,
}

impl MessageHandler {
    pub fn new(db: DatabaseConnection) -> Self {
        MessageHandler { db }
    }
    
    pub async fn handle_message(&self, api: Bot, message: Message) -> Result<()> {
        if let Some(text) = &message.text {
            println!("收到消息: {}", text);
            
            // 检查是否是命令
            if text.starts_with('/') {
                self.handle_command(api, &message, text).await?;
            } else {
                // 检查关键词匹配
                self.handle_keyword_reply(api, &message, text).await?;
            }
        }
        
        Ok(())
    }
    
    async fn handle_command(&self, api: Bot, message: &Message, _text: &str) -> Result<()> {
        // 通过实体 offset 来判断命令类型
        if let Some(command) = self.get_bot_command_from_entities(message)? {
            match command.as_str() {
                "/add" => {
                    // 通过实体 offset 获取命令后的内容
                    let content = self.get_content_after_command(message, &command)?;
                    
                    if content.is_empty() {
                        self.send_reply(api, message.chat.id, "用法: /add &lt;关键词&gt; &lt;回复内容&gt;").await?;
                        return Ok(());
                    }
                    
                    // 找到第一个空格或换行符的位置
                    let delimiter_pos = content.find(|c| c == ' ' || c == '\n');
                    if let Some(pos) = delimiter_pos {
                        let keywords = content[..pos].to_string();
                        let reply_content = content[pos + 1..].to_string();
                        
                        if keywords.is_empty() || reply_content.is_empty() {
                            self.send_reply(api, message.chat.id, "用法: /add &lt;关键词&gt; &lt;回复内容&gt;").await?;
                            return Ok(());
                        }
                        
                        // 检查用户是否为管理员
                        if !self.is_admin(&api, message).await? {
                            self.send_reply(api, message.chat.id, "只有管理员才能使用此命令").await?;
                            return Ok(());
                        }
                        
                        // 处理回复内容，检查消息实体中的 code 类型并用 <code> 标签包裹
                        let processed_reply = self.process_reply_with_entities(&reply_content, &message).await?;
                        
                        // 保存到数据库
                        let is_updated = self.add_keyword_reply(message.chat.id, keywords.clone(), processed_reply).await?;
                        let message_text = if is_updated {
                            format!("关键词 <code>{}</code> 的回复内容已更新成功！", keywords)
                        } else {
                            format!("关键词 <code>{}</code> 回复已添加成功！", keywords)
                        };
                        self.send_reply(api, message.chat.id, &message_text).await?;
                    } else {
                        self.send_reply(api, message.chat.id, "用法: /add &lt;关键词&gt; &lt;回复内容&gt;").await?;
                        return Ok(());
                    }
                }
                "/del" => {
                    let content = self.get_content_after_command(message, &command)?;
                    if content.is_empty() {
                        self.send_reply(api, message.chat.id, "用法: /del &lt;关键词&gt;").await?;
                        return Ok(());
                    }
                    
                    // 检查用户是否为管理员
                    if !self.is_admin(&api, message).await? {
                        self.send_reply(api, message.chat.id, "只有管理员才能使用此命令").await?;
                        return Ok(());
                    }
                    
                    let keywords = content.trim().to_string();
                    
                    // 删除关键词
                    match self.delete_keyword_reply(message.chat.id, keywords.clone()).await {
                        Ok(true) => {
                            self.send_reply(api, message.chat.id, &format!("关键词 <code>{}</code> 已删除成功！", keywords)).await?;
                        }
                        Ok(false) => {
                            self.send_reply(api, message.chat.id, &format!("未找到关键词 <code>{}</code>", keywords)).await?;
                        }
                        Err(e) => {
                            eprintln!("删除关键词时出错: {}", e);
                            self.send_reply(api, message.chat.id, "删除关键词时出错，请稍后重试").await?;
                        }
                    }
                }
                "/all" => {
                    self.show_all_keywords(api, message.chat.id).await?;
                }
                "/del_all" => {
                    // 检查用户是否为管理员
                    if !self.is_admin(&api, message).await? {
                        self.send_reply(api, message.chat.id, "只有管理员才能使用此命令").await?;
                        return Ok(());
                    }
                    
                    // 删除所有关键词
                    match self.delete_all_keywords(message.chat.id).await {
                        Ok(count) => {
                            if count > 0 {
                                self.send_reply(api, message.chat.id, &format!("已删除 {} 个关键词！", count)).await?;
                            } else {
                                self.send_reply(api, message.chat.id, "当前群组没有设置任何关键词").await?;
                            }
                        }
                        Err(e) => {
                            eprintln!("删除所有关键词时出错: {}", e);
                            self.send_reply(api, message.chat.id, "删除关键词时出错，请稍后重试").await?;
                        }
                    }
                }
                "/help" => {
                    self.send_reply(api, message.chat.id, "可用命令:\n/add &lt;关键词&gt; &lt;回复内容&gt; - 添加关键词回复（仅管理员）\n/del &lt;关键词&gt; - 删除关键词回复（仅管理员）\n/del_all - 删除当前群组的所有关键词（仅管理员）\n/all - 查看当前群组的所有关键词\n/help - 显示帮助信息").await?;
                }
                _ => {
                    // 未知命令，不进行回应
                }
            }
        } else {
            // 没有找到命令实体，不进行回应
        }
        
        Ok(())
    }
    
    async fn handle_keyword_reply(&self, api: Bot, message: &Message, text: &str) -> Result<()> {
        // 查询数据库中的关键词匹配
        let replies = GroupReplyEntity::find()
            .filter(group_reply::Column::GroupId.eq(message.chat.id))
            .all(&self.db)
            .await?;
        
        for reply in replies {
            if text.contains(&reply.keywords) {
                self.send_reply(api, message.chat.id, &reply.reply).await?;
                return Ok(());
            }
        }
        
        // 如果没有匹配的关键词，则忽略消息
        Ok(())
    }
    
    async fn is_admin(&self, api: &Bot, message: &Message) -> Result<bool> {
        if let Some(from) = &message.from {
            let chat_id = message.chat.id;
            let user_id = from.id;
            
            let params = GetChatMemberParams::builder()
                .chat_id(chat_id)
                .user_id(user_id)
                .build();
            
            match api.get_chat_member(&params).await {
                Ok(response) => {
                    let member = &response.result;
                    Ok(matches!(member, ChatMember::Administrator(_) | ChatMember::Creator(_)))
                }
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
    
    async fn add_keyword_reply(&self, group_id: i64, keywords: String, reply: String) -> Result<bool> {
        // 先检查是否已存在相同的关键词
        let existing_reply = GroupReplyEntity::find()
            .filter(group_reply::Column::GroupId.eq(group_id))
            .filter(group_reply::Column::Keywords.eq(&keywords))
            .one(&self.db)
            .await?;
        
        if let Some(existing) = existing_reply {
            // 如果存在，则更新回复内容
            let mut active_model: group_reply::ActiveModel = existing.into();
            active_model.reply = Set(reply);
            active_model.update(&self.db).await?;
            Ok(true) // 返回 true 表示更新
        } else {
            // 如果不存在，则插入新记录
            let new_reply = group_reply::ActiveModel {
                group_id: Set(group_id),
                keywords: Set(keywords),
                reply: Set(reply),
            };
            
            new_reply.insert(&self.db).await?;
            Ok(false) // 返回 false 表示添加
        }
    }
    
    async fn delete_keyword_reply(&self, group_id: i64, keywords: String) -> Result<bool> {
        let result = GroupReplyEntity::delete_many()
            .filter(group_reply::Column::GroupId.eq(group_id))
            .filter(group_reply::Column::Keywords.eq(keywords))
            .exec(&self.db)
            .await?;
        
        Ok(result.rows_affected > 0)
    }
    
    async fn delete_all_keywords(&self, group_id: i64) -> Result<u64> {
        let result = GroupReplyEntity::delete_many()
            .filter(group_reply::Column::GroupId.eq(group_id))
            .exec(&self.db)
            .await?;
        
        Ok(result.rows_affected)
    }
    
    async fn show_all_keywords(&self, api: Bot, chat_id: i64) -> Result<()> {
        // 查询当前群组的所有关键词
        let replies = GroupReplyEntity::find()
            .filter(group_reply::Column::GroupId.eq(chat_id))
            .all(&self.db)
            .await?;
        
        if replies.is_empty() {
            self.send_reply(api, chat_id, "当前群组还没有设置任何关键词回复。").await?;
        } else {
            let mut message = "<b>当前群组的关键词列表:</b>\n\n".to_string();
            for reply in replies.iter() {
                message.push_str(&format!("<code>{}</code>\n", reply.keywords));
            }
            self.send_reply(api, chat_id, &message).await?;
        }
        
        Ok(())
    }
    
    fn get_bot_command_from_entities(&self, message: &Message) -> Result<Option<String>> {
        if let Some(entities) = &message.entities {
            for entity in entities {
                if entity.type_field == MessageEntityType::BotCommand {
                    let message_text = message.text.as_deref().unwrap_or("");
                    let start = entity.offset as usize;
                    let end = start + entity.length as usize;
                    
                    // 将 UTF-16 偏移量转换为 UTF-8 偏移量
                    let utf8_start = self.utf16_to_utf8_offset(message_text, start)?;
                    let utf8_end = self.utf16_to_utf8_offset(message_text, end)?;
                    
                    if utf8_start < message_text.len() && utf8_end <= message_text.len() {
                        let command = &message_text[utf8_start..utf8_end];
                        return Ok(Some(command.to_string()));
                    }
                }
            }
        }
        Ok(None)
    }
    
    fn get_content_after_command(&self, message: &Message, command: &str) -> Result<String> {
        let message_text = message.text.as_deref().unwrap_or("");
        
        // 找到命令在消息中的位置
        if let Some(command_pos) = message_text.find(command) {
            let content_start = command_pos + command.len();
            let content = &message_text[content_start..];
            Ok(content.trim_start().to_string())
        } else {
            Ok(String::new())
        }
    }
    
    async fn process_reply_with_entities(&self, reply_content: &str, message: &Message) -> Result<String> {
        // 检查消息是否有实体
        if let Some(entities) = &message.entities {
            let message_text = message.text.as_deref().unwrap_or("");
            
            // 找到回复内容在消息中的位置
            let reply_start_in_message = self.find_reply_start_in_message(message_text, reply_content)?;
            
            let mut processed_content = reply_content.to_string();
            
            // 将回复内容转换为 UTF-16 编码以正确处理偏移量
            let reply_utf16_chars: Vec<u16> = reply_content.encode_utf16().collect();
            
            // 计算回复内容在消息中的 UTF-16 起始位置
            let reply_start_utf16 = self.utf8_to_utf16_offset(message_text, reply_start_in_message)?;
            let reply_end_utf16 = reply_start_utf16 + reply_utf16_chars.len();
            
            // 按 offset 排序实体，从后往前处理以避免偏移量问题
            let mut sorted_entities: Vec<_> = entities.iter().collect();
            sorted_entities.sort_by_key(|entity| entity.offset);
            sorted_entities.reverse();
            
            for entity in sorted_entities {
                if entity.type_field == MessageEntityType::Code {
                    let entity_start = entity.offset as usize;
                    let entity_end = entity_start + entity.length as usize;
                    
                    // 检查实体是否在回复内容范围内
                    if entity_start >= reply_start_utf16 && entity_end <= reply_end_utf16 {
                        // 将实体偏移量转换为回复内容内的偏移量
                        let relative_start = entity_start - reply_start_utf16;
                        let relative_end = entity_end - reply_start_utf16;
                        
                        // 确保索引在回复内容的 UTF-16 字符数组的有效范围内
                        if relative_start < reply_utf16_chars.len() && relative_end <= reply_utf16_chars.len() {
                            // 将 UTF-16 字符转换回字符串
                            let code_utf16: Vec<u16> = reply_utf16_chars[relative_start..relative_end].to_vec();
                            let code_text = String::from_utf16(&code_utf16)
                                .unwrap_or_else(|_| "".to_string());
                            
                            // 找到在原始 UTF-8 字符串中的位置
                            let utf8_start = self.utf16_to_utf8_offset(&processed_content, relative_start)?;
                            let utf8_end = self.utf16_to_utf8_offset(&processed_content, relative_end)?;
                            
                            if utf8_start < processed_content.len() && utf8_end <= processed_content.len() {
                                let wrapped_code = format!("<code>{}</code>", code_text);
                                processed_content.replace_range(utf8_start..utf8_end, &wrapped_code);
                            }
                        }
                    }
                }
            }
            
            Ok(processed_content)
        } else {
            // 如果没有实体，直接返回原内容
            Ok(reply_content.to_string())
        }
    }
    
    fn find_reply_start_in_message(&self, message_text: &str, reply_content: &str) -> Result<usize> {
        // 找到回复内容在消息中的起始位置
        if let Some(pos) = message_text.find(reply_content) {
            Ok(pos)
        } else {
            // 如果找不到，返回 0（可能是消息格式不匹配）
            Ok(0)
        }
    }
    
    fn utf8_to_utf16_offset(&self, text: &str, utf8_offset: usize) -> Result<usize> {
        let mut utf16_count = 0;
        let mut utf8_count = 0;
        
        for ch in text.chars() {
            if utf8_count >= utf8_offset {
                break;
            }
            
            utf8_count += ch.len_utf8();
            utf16_count += ch.len_utf16();
        }
        
        Ok(utf16_count)
    }
    
    fn utf16_to_utf8_offset(&self, text: &str, utf16_offset: usize) -> Result<usize> {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        
        for ch in text.chars() {
            if utf16_count >= utf16_offset {
                break;
            }
            
            utf8_offset += ch.len_utf8();
            utf16_count += ch.len_utf16();
        }
        
        Ok(utf8_offset)
    }
    
    async fn send_reply(&self, api: Bot, chat_id: i64, text: &str) -> Result<()> {
        let reply_params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(text)
            .parse_mode(ParseMode::Html)
            .build();
            
        if let Err(e) = api.send_message(&reply_params).await {
            eprintln!("发送回复时出错: {}", e);
        }
        
        Ok(())
    }
}