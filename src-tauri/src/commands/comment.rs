use crate::comment::{self, CommentPage};

#[tauri::command]
pub async fn get_comments(aid: i64, offset: Option<String>) -> Result<CommentPage, String> {
    comment::fetch_comments(aid, offset).await
}

#[tauri::command]
pub async fn get_sub_replies(
    aid: i64,
    root_rpid: i64,
) -> Result<Vec<crate::comment::Reply>, String> {
    crate::comment::fetch_sub_replies(aid, root_rpid).await
}
