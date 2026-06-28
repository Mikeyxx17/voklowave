//! 私聊数据模型。

/// 发起私聊请求
#[derive(serde::Deserialize)]
pub struct StartDmInput {
    pub user_id: i32,
}

/// 私聊列表项（返回给前端）
#[derive(serde::Serialize)]
pub struct DmConversationItem {
    pub conversation_id: i32,
    pub other_user_id: i32,
    pub other_username: String,
    pub other_display_name: Option<String>,
    pub other_avatar_url: Option<String>,
    pub last_message: Option<String>,
    pub last_message_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 私聊消息
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DmMessage {
    pub id: Option<i32>,
    #[serde(default)]
    pub conversation_id: i32,
    #[serde(default)]
    pub sender_id: i32,
    pub sender_username: Option<String>,
    pub content: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
