use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "group_reply")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub group_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub keywords: String,
    pub reply: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// 导出实体模块
pub mod group_reply {
    pub use super::*;
}