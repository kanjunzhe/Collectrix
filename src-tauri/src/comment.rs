use crate::bilibili;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
struct ReplyMainResponse {
    code: i64,
    message: String,
    data: Option<ReplyMainData>,
}

#[derive(Debug, Deserialize)]
struct ReplyMainData {
    replies: Option<Vec<Reply>>,
    top: Option<Box<TopReply>>,
    cursor: Cursor,
}

#[derive(Debug, Deserialize)]
struct TopReply {
    upper: Option<Reply>,
}

#[derive(Debug, Deserialize)]
struct Cursor {
    is_end: bool,
    pagination_reply: Option<PaginationReply>,
}

#[derive(Debug, Deserialize)]
struct PaginationReply {
    next_offset: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Reply {
    pub rpid: i64,
    pub oid: i64,
    pub mid: i64,
    pub root: i64,
    pub parent: i64,
    pub count: i64,
    pub rcount: i64,
    pub like: i64,
    pub ctime: i64,
    pub content: ReplyContent,
    pub member: ReplyMember,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReplyContent {
    pub message: String,
    #[serde(default)]
    pub emote: Option<std::collections::HashMap<String, ReplyEmote>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReplyEmote {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub width: Option<i64>,
    #[serde(default)]
    pub height: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReplyMember {
    pub uname: String,
    pub avatar: String,
}

#[derive(Debug, Serialize)]
pub struct CommentPage {
    pub replies: Vec<Reply>,
    pub top: Option<Reply>,
    pub next_offset: Option<String>,
    pub is_end: bool,
}

#[derive(Debug, Deserialize)]
struct SubReplyResponse {
    code: i64,
    message: String,
    data: Option<SubReplyData>,
}

#[derive(Debug, Deserialize)]
struct SubReplyData {
    replies: Option<Vec<Reply>>,
    page: SubReplyPage,
}

#[derive(Debug, Deserialize)]
struct SubReplyPage {
    count: i64,
}

pub async fn fetch_sub_replies(aid: i64, root_rpid: i64) -> Result<Vec<Reply>, String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .map_err(|e| e.to_string())?;

    let mut params: std::collections::BTreeMap<String, String> = Default::default();
    params.insert("oid".into(), aid.to_string());
    params.insert("type".into(), "1".into());
    params.insert("root".into(), root_rpid.to_string());
    params.insert("ps".into(), "20".into());
    params.insert("pn".into(), "1".into());

    let (img_key, sub_key) = crate::bilibili::fetch_wbi_keys(&client).await?;
    let mixin_key = crate::bilibili::get_mixin_key(&img_key, &sub_key);
    crate::bilibili::sign_params(&mut params, &mixin_key);

    let resp: SubReplyResponse = client
        .get("https://api.bilibili.com/x/v2/reply/reply")
        .query(&params)
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if resp.code != 0 {
        return Err(format!("子回复接口错误: {} - {}", resp.code, resp.message));
    }
    Ok(resp.data.and_then(|d| d.replies).unwrap_or_default())
}

pub async fn fetch_comments(aid: i64, offset: Option<String>) -> Result<CommentPage, String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .map_err(|e| e.to_string())?;

    let (img_key, sub_key) = bilibili::fetch_wbi_keys(&client).await?;
    let mixin_key = bilibili::get_mixin_key(&img_key, &sub_key);

    let mut params: BTreeMap<String, String> = BTreeMap::new();
    params.insert("oid".into(), aid.to_string());
    params.insert("type".into(), "1".into()); // 1 = 视频
    params.insert("mode".into(), "3".into()); // 3 = 热门排序
    params.insert("plat".into(), "1".into());
    params.insert("web_location".into(), "1315875".into());

    if let Some(off) = offset {
        let pagination = format!(r#"{{"offset":"{}"}}"#, off);
        params.insert("pagination_str".into(), pagination);
    } else {
        params.insert("pagination_str".into(), r#"{"offset":""}"#.into());
    }

    bilibili::sign_params(&mut params, &mixin_key);

    let resp: ReplyMainResponse = client
        .get("https://api.bilibili.com/x/v2/reply/wbi/main")
        .query(&params)
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if resp.code != 0 {
        return Err(format!("评论接口错误: {} - {}", resp.code, resp.message));
    }

    let data = resp.data.ok_or_else(|| "评论 data 为空".to_string())?;
    let replies = data.replies.unwrap_or_default();
    let top = data.top.and_then(|t| t.upper);
    let next_offset = data
        .cursor
        .pagination_reply
        .and_then(|p| p.next_offset)
        .filter(|s| !s.is_empty());

    Ok(CommentPage {
        replies,
        top,
        next_offset,
        is_end: data.cursor.is_end,
    })
}
