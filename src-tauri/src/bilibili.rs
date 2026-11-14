// use crate::settings::AppSettings;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use url::form_urlencoded;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FavFolder {
    pub id: i64,
    pub fid: i64,
    pub title: String,
    pub media_count: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FavItem {
    pub id: i64,
    pub bvid: String,
    pub title: String,
    pub cover: String,
    pub upper: FavUpper,
    pub intro: String,
    pub duration: i64,
    pub attr: i64, // 0=正常, 1=失效
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FavUpper {
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct FolderListResponse {
    code: i64,
    message: String,
    data: Option<FolderListData>,
}

#[derive(Debug, Deserialize)]
struct FolderListData {
    list: Option<Vec<FavFolder>>,
}

#[derive(Debug, Deserialize)]
struct FavResourceResponse {
    code: i64,
    message: String,
    data: Option<FavResourceData>,
}

#[derive(Debug, Deserialize)]
struct FavResourceData {
    info: FavResourceInfo,
    medias: Option<Vec<FavItem>>,
    has_more: bool,
}

#[derive(Debug, Deserialize)]
struct FavResourceInfo {
    media_count: i64,
}

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

#[derive(Debug, Deserialize)]
struct NavResponse {
    data: NavData,
}

#[derive(Debug, Deserialize)]
struct NavData {
    wbi_img: WbiImg,
}

#[derive(Debug, Deserialize)]
struct WbiImg {
    img_url: String,
    sub_url: String,
}

#[derive(Debug, Deserialize)]
struct ViewResponse<T> {
    code: i64,
    message: String,
    data: Option<T>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BiliVideoData {
    pub aid: i64,
    pub bvid: String,
    pub cid: i64,
    pub title: String,
    pub pic: String,
    pub desc: String,
    pub duration: i64,
    pub owner: BiliOwner,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BiliOwner {
    pub name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct BulkVideo {
    pub platform: String,
    pub platform_id: String,
    pub title: String,
    pub author: Option<String>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub duration: Option<i64>,
    pub original_url: String,
    pub aid: Option<i64>,
    pub cid: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct QrCodeInfo {
    pub url: String,
    pub qrcode_key: String,
}

#[derive(Debug, Serialize)]
pub struct QrLoginStatus {
    pub code: i64,
    pub message: String,
    pub logged_in: bool,
}

#[derive(Debug, Serialize)]
pub struct BiliLoginInfo {
    pub is_login: bool,
    pub mid: Option<i64>,
    pub uname: Option<String>,
}

const BILI_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36";

pub async fn generate_qr_code() -> Result<QrCodeInfo, String> {
    let client = Client::builder()
        .user_agent(BILI_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = client
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate")
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    if json["code"].as_i64() != Some(0) {
        return Err(format!("生成二维码失败: {}", json["message"]));
    }
    let url = json["data"]["url"]
        .as_str()
        .ok_or("二维码 URL 为空")?
        .to_string();
    let qrcode_key = json["data"]["qrcode_key"]
        .as_str()
        .ok_or("二维码 key 为空")?
        .to_string();
    Ok(QrCodeInfo { url, qrcode_key })
}

pub async fn poll_qr_code(qrcode_key: &str) -> Result<(QrLoginStatus, Option<String>), String> {
    let client = Client::builder()
        .user_agent(BILI_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/poll")
        .query(&[("qrcode_key", qrcode_key)])
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut cookie = extract_set_cookie(&resp);
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let code = json["data"]["code"].as_i64().unwrap_or(-1);
    let message = json["data"]["message"]
        .as_str()
        .unwrap_or("")
        .to_string();

    if code == 0 {
        if let Some(url) = json["data"]["url"].as_str() {
            cookie = merge_cookie(cookie, cookie_from_url(url));
        }
    }

    Ok((
        QrLoginStatus {
            code,
            message,
            logged_in: code == 0,
        },
        if code == 0 { cookie } else { None },
    ))
}

fn cookie_from_url(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    let query = parsed.query()?;
    let allowed = [
        "SESSDATA",
        "bili_jct",
        "DedeUserID",
        "DedeUserID__ckMd5",
        "sid",
        "buvid3",
        "buvid4",
    ];
    let mut cookies = Vec::new();
    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        if allowed.contains(&key) {
            cookies.push(format!("{key}={value}"));
        }
    }
    if cookies.is_empty() {
        None
    } else {
        Some(cookies.join("; "))
    }
}

fn merge_cookie(base: Option<String>, extra: Option<String>) -> Option<String> {
    let mut pairs: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    if let Some(base) = base {
        for value in base.split(';').map(str::trim) {
            if let Some((name, value)) = value.split_once('=') {
                pairs.insert(name.to_string(), value.to_string());
            }
        }
    }
    // URL 中携带的 Cookie 通常更完整，覆盖同名 Set-Cookie。
    if let Some(extra) = extra {
        for value in extra.split(';').map(str::trim) {
            if let Some((name, value)) = value.split_once('=') {
                pairs.insert(name.to_string(), value.to_string());
            }
        }
    }
    if pairs.is_empty() {
        None
    } else {
        Some(
            pairs
                .into_iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect::<Vec<_>>()
                .join("; "),
        )
    }
}

pub async fn check_login(cookie: &str) -> Result<BiliLoginInfo, String> {
    if cookie.is_empty() {
        return Ok(BiliLoginInfo {
            is_login: false,
            mid: None,
            uname: None,
        });
    }
    let client = client_with_cookie(cookie)?;
    let json: serde_json::Value = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let data = &json["data"];
    Ok(BiliLoginInfo {
        is_login: data["isLogin"].as_bool().unwrap_or(false),
        mid: data["mid"].as_i64(),
        uname: data["uname"].as_str().map(String::from),
    })
}

/// 从合集/视频页解析合集内所有视频。传入合集内任意一集的 URL 即可。
pub async fn fetch_collection(url: &str) -> Result<Vec<BulkVideo>, String> {
    let bvid = extract_bvid(url).ok_or("无法从 URL 中识别 Bilibili BV 号")?;
    let client = Client::builder()
        .user_agent(BILI_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;

    let (img_key, sub_key) = fetch_wbi_keys(&client).await?;
    let mixin_key = get_mixin_key(&img_key, &sub_key);
    let mut params = BTreeMap::new();
    params.insert("bvid".into(), bvid);
    sign_params(&mut params, &mixin_key);

    let json: serde_json::Value = client
        .get("https://api.bilibili.com/x/web-interface/wbi/view")
        .query(&params)
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if json["code"].as_i64() != Some(0) {
        return Err(format!("合集接口错误: {}", json["message"]));
    }
    let data = &json["data"];
    let owner = data["owner"]["name"].as_str().map(String::from);
    let parent_cover = data["pic"].as_str().map(String::from);
    let parent_desc = data["desc"].as_str().map(String::from);
    let parent_duration = data["duration"].as_i64();

    let mut items = Vec::new();
    if let Some(sections) = data["ugc_season"]["sections"].as_array() {
        for section in sections {
            if let Some(episodes) = section["episodes"].as_array() {
                for episode in episodes {
                    let ep_bvid = episode["bvid"].as_str().unwrap_or("").to_string();
                    if ep_bvid.is_empty() {
                        continue;
                    }
                    let arc = &episode["arc"];
                    let title = episode["title"]
                        .as_str()
                        .or_else(|| arc["title"].as_str())
                        .unwrap_or("")
                        .to_string();
                    let cover = arc["pic"]
                        .as_str()
                        .map(String::from)
                        .or_else(|| parent_cover.clone());
                    let description = arc["desc"]
                        .as_str()
                        .map(String::from)
                        .or_else(|| parent_desc.clone());
                    let duration = arc["duration"].as_i64().or(parent_duration);
                    let author = arc["author"]["name"]
                        .as_str()
                        .map(String::from)
                        .or_else(|| owner.clone());
                    items.push(BulkVideo {
                        platform: "bilibili".into(),
                        original_url: format!("https://www.bilibili.com/video/{ep_bvid}"),
                        platform_id: ep_bvid,
                        title,
                        author,
                        cover_url: cover,
                        description,
                        duration,
                        aid: episode["aid"].as_i64().or_else(|| arc["aid"].as_i64()),
                        cid: episode["cid"].as_i64().or_else(|| arc["cid"].as_i64()),
                    });
                }
            }
        }
    }

    if items.is_empty() {
        return Err("这个链接不是合集，或合集里没有视频".into());
    }
    Ok(items)
}

/// 获取当前账号的「稍后再看」列表。
pub async fn fetch_watch_later(cookie: &str) -> Result<Vec<BulkVideo>, String> {
    if cookie.is_empty() {
        return Err("请先在设置中填写 B 站 Cookie".into());
    }
    let client = client_with_cookie(cookie)?;
    let json: serde_json::Value = client
        .get("https://api.bilibili.com/x/v2/history/toview")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    if json["code"].as_i64() != Some(0) {
        let code = json["code"].as_i64().unwrap_or_default();
        let message = json["message"].as_str().unwrap_or("未知错误");
        if code == -101 || message.contains("未登录") {
            return Err("B站登录状态已失效，请在设置中重新扫码登录".into());
        }
        return Err(format!("获取稍后再看失败: {code} - {message}"));
    }

    let list = json["data"]["list"].as_array().cloned().unwrap_or_default();
    Ok(list
        .into_iter()
        .filter_map(|item| {
            let bvid = item["bvid"].as_str()?.to_string();
            Some(BulkVideo {
                platform: "bilibili".into(),
                original_url: format!("https://www.bilibili.com/video/{bvid}"),
                platform_id: bvid,
                title: item["title"].as_str().unwrap_or("").to_string(),
                author: item["owner"]["name"].as_str().map(String::from),
                cover_url: item["pic"].as_str().map(String::from),
                description: item["desc"].as_str().map(String::from),
                duration: item["duration"].as_i64(),
                aid: item["aid"].as_i64(),
                cid: item["cid"].as_i64(),
            })
        })
        .collect())
}

fn is_login_cookie_name(name: &str) -> bool {
    matches!(
        name,
        "SESSDATA"
            | "bili_jct"
            | "DedeUserID"
            | "DedeUserID__ckMd5"
            | "sid"
            | "buvid3"
            | "buvid4"
    )
}

fn extract_set_cookie(resp: &reqwest::Response) -> Option<String> {
    let mut values = Vec::new();
    for value in resp.headers().get_all(reqwest::header::SET_COOKIE).iter() {
        if let Ok(text) = value.to_str() {
            // 有些服务器会把多个 Set-Cookie 合并到同一行。
            for part in text.split(',') {
                let Some(first) = part.split(';').next() else {
                    continue;
                };
                let first = first.trim();
                let Some((name, _)) = first.split_once('=') else {
                    continue;
                };
                if is_login_cookie_name(name) {
                    values.push(first.to_string());
                }
            }
        }
    }
    if values.is_empty() {
        None
    } else {
        Some(values.join("; "))
    }
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub async fn fetch_wbi_keys(client: &Client) -> Result<(String, String), String> {
    let resp: NavResponse = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let img_key = extract_key(&resp.data.wbi_img.img_url);
    let sub_key = extract_key(&resp.data.wbi_img.sub_url);
    Ok((img_key, sub_key))
}

fn extract_key(url: &str) -> String {
    url.rsplit('/')
        .next()
        .unwrap_or(url)
        .trim_end_matches(".png")
        .to_string()
}

pub fn get_mixin_key(img_key: &str, sub_key: &str) -> String {
    let raw = format!("{}{}", img_key, sub_key);
    MIXIN_KEY_ENC_TAB
        .iter()
        .map(|&i| raw.as_bytes().get(i).copied().unwrap_or(b'\0') as char)
        .take(32)
        .collect()
}

pub fn sign_params(params: &mut BTreeMap<String, String>, mixin_key: &str) {
    let wts = now_ts().to_string();
    params.insert("wts".into(), wts);

    let query: String = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(params.iter())
        .finish();

    let to_hash = format!("{}{}", query, mixin_key);
    let w_rid = format!("{:x}", md5::compute(to_hash));
    params.insert("w_rid".into(), w_rid);
}

pub async fn fetch_video_info(bvid: &str) -> Result<BiliVideoData, String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .map_err(|e| e.to_string())?;

    let (img_key, sub_key) = fetch_wbi_keys(&client).await?;
    let mixin_key = get_mixin_key(&img_key, &sub_key);

    let mut params = BTreeMap::new();
    params.insert("bvid".into(), bvid.to_string());
    sign_params(&mut params, &mixin_key);

    let resp: ViewResponse<BiliVideoData> = client
        .get("https://api.bilibili.com/x/web-interface/wbi/view")
        .query(&params)
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if resp.code != 0 {
        return Err(format!("B站接口返回错误: {} - {}", resp.code, resp.message));
    }
    resp.data.ok_or_else(|| "B站接口 data 为空".to_string())
}

/// 从 B站 URL 中提取 BV 号
#[derive(Debug, Serialize, Clone)]
pub struct BiliSearchItem {
    pub bvid: String,
    pub title: String,
    pub author: Option<String>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub duration: Option<i64>,
}

fn strip_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
}

fn parse_duration_text(value: &str) -> Option<i64> {
    if let Ok(seconds) = value.parse::<i64>() {
        return Some(seconds);
    }
    let parts: Vec<&str> = value.split(':').collect();
    let mut seconds = 0i64;
    for part in parts {
        seconds = seconds * 60 + part.parse::<i64>().ok()?;
    }
    Some(seconds)
}

/// 按关键词搜索 B 站视频，用于本地文件匹配元数据。
pub async fn search_videos(keyword: &str, cookie: &str) -> Result<Vec<BiliSearchItem>, String> {
    let keyword = keyword.trim();
    if keyword.is_empty() {
        return Ok(Vec::new());
    }
    let client = if cookie.is_empty() {
        Client::builder()
            .user_agent(BILI_USER_AGENT)
            .build()
            .map_err(|e| e.to_string())?
    } else {
        client_with_cookie(cookie)?
    };

    let (img_key, sub_key) = fetch_wbi_keys(&client).await?;
    let mixin_key = get_mixin_key(&img_key, &sub_key);
    let mut params = BTreeMap::new();
    params.insert("keyword".into(), keyword.to_string());
    params.insert("search_type".into(), "video".into());
    params.insert("page".into(), "1".into());
    params.insert("page_size".into(), "12".into());
    sign_params(&mut params, &mixin_key);

    let json: serde_json::Value = client
        .get("https://api.bilibili.com/x/web-interface/wbi/search/type")
        .query(&params)
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let code = json["code"].as_i64().unwrap_or_default();
    if code != 0 {
        let message = json["message"].as_str().unwrap_or("未知错误");
        return Err(format!("B 站搜索失败: {code} - {message}"));
    }

    Ok(json["data"]["result"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            let bvid = item["bvid"].as_str()?.to_string();
            if bvid.is_empty() {
                return None;
            }
            let cover = item["pic"].as_str().map(|url| {
                if url.starts_with("//") {
                    format!("https:{url}")
                } else {
                    url.to_string()
                }
            });
            Some(BiliSearchItem {
                bvid,
                title: strip_html_tags(item["title"].as_str().unwrap_or("")),
                author: item["author"].as_str().map(String::from),
                cover_url: cover,
                description: item["description"].as_str().map(strip_html_tags),
                duration: item["duration"]
                    .as_str()
                    .and_then(parse_duration_text)
                    .or_else(|| item["duration"].as_i64()),
            })
        })
        .collect())
}

pub fn extract_bvid(url: &str) -> Option<String> {
    if let Some(idx) = url.find("BV") {
        let rest = &url[idx..];
        let bvid: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();
        if bvid.len() >= 10 {
            return Some(bvid);
        }
    }
    None
}

fn client_with_cookie(cookie: &str) -> Result<Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::COOKIE,
        reqwest::header::HeaderValue::from_str(cookie).map_err(|e| e.to_string())?,
    );
    headers.insert(
        reqwest::header::REFERER,
        reqwest::header::HeaderValue::from_static("https://www.bilibili.com/"),
    );
    headers.insert(
        reqwest::header::ORIGIN,
        reqwest::header::HeaderValue::from_static("https://www.bilibili.com"),
    );
    Client::builder()
        .user_agent(BILI_USER_AGENT)
        .default_headers(headers)
        .build()
        .map_err(|e| e.to_string())
}

pub async fn fetch_fav_folders(cookie: &str, up_mid: i64) -> Result<Vec<FavFolder>, String> {
    if cookie.is_empty() {
        return Err("请先在设置中填写 B 站 Cookie".into());
    }
    let client = client_with_cookie(cookie)?;
    let resp: FolderListResponse = client
        .get("https://api.bilibili.com/x/v3/fav/folder/created/list-all")
        .query(&[("up_mid", up_mid.to_string())])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if resp.code != 0 {
        return Err(format!("获取收藏夹失败: {} - {}", resp.code, resp.message));
    }
    Ok(resp.data.and_then(|d| d.list).unwrap_or_default())
}

pub async fn fetch_fav_videos(
    cookie: &str,
    media_id: i64,
    page: i64,
) -> Result<(Vec<FavItem>, bool), String> {
    if cookie.is_empty() {
        return Err("请先在设置中填写 B 站 Cookie".into());
    }
    let client = client_with_cookie(cookie)?;
    let resp: FavResourceResponse = client
        .get("https://api.bilibili.com/x/v3/fav/resource/list")
        .query(&[
            ("media_id", media_id.to_string()),
            ("pn", page.to_string()),
            ("ps", "20".to_string()),
            ("keyword", "".to_string()),
            ("order", "mtime".to_string()),
            ("type", "0".to_string()),
            ("tid", "0".to_string()),
            ("platform", "web".to_string()),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if resp.code != 0 {
        return Err(format!(
            "获取收藏夹内容失败: {} - {}",
            resp.code, resp.message
        ));
    }
    let data = resp.data.ok_or("收藏夹 data 为空")?;
    let has_more = data.has_more;
    let medias = data.medias.unwrap_or_default();
    Ok((medias, has_more))
}

// 弹幕 XML 抓取 + 解析
pub async fn fetch_danmaku_xml(cid: i64) -> Result<Vec<DanmakuItem>, String> {
    let url = format!("https://comment.bilibili.com/{cid}.xml");
    // B 站弹幕 CDN 返回的是 raw DEFLATE（没有 zlib 头），
    // reqwest 的好心自动解码会报 error decoding response body，所以这里关闭自动解码并手动处理。
    let resp = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .build()
        .map_err(|e| e.to_string())?
        .get(&url)
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("获取弹幕失败 HTTP {status}"));
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let text = decode_danmaku_payload(&bytes)?;

    parse_danmaku_xml(&text)
}

fn decode_danmaku_payload(bytes: &[u8]) -> Result<String, String> {
    fn decode_text(bytes: &[u8]) -> String {
        match String::from_utf8(bytes.to_vec()) {
            Ok(text) => text,
            Err(_) => {
                let (text, _, _) = encoding_rs::GBK.decode(bytes);
                text.into_owned()
            }
        }
    }

    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
        if text.trim_start().starts_with('<') {
            return Ok(text);
        }
    }

    use std::io::Read;
    let mut decoded = Vec::new();
    if flate2::read::DeflateDecoder::new(bytes)
        .read_to_end(&mut decoded)
        .is_ok()
        && !decoded.is_empty()
    {
        return Ok(decode_text(&decoded));
    }

    let mut decoded = Vec::new();
    if flate2::read::ZlibDecoder::new(bytes)
        .read_to_end(&mut decoded)
        .is_ok()
        && !decoded.is_empty()
    {
        return Ok(decode_text(&decoded));
    }

    let mut decoded = Vec::new();
    if flate2::read::GzDecoder::new(bytes)
        .read_to_end(&mut decoded)
        .is_ok()
        && !decoded.is_empty()
    {
        return Ok(decode_text(&decoded));
    }

    Err("弹幕响应解码失败".into())
}

#[derive(serde::Serialize, Clone)]
pub struct DanmakuItem {
    pub time: f64, // 出现时间（秒）
    pub mode: i32, // 1/2/3=滚动, 4=底部, 5=顶部
    pub fontsize: i32,
    pub color: u32, // 十进制颜色
    pub text: String,
}

fn parse_danmaku_xml(xml: &str) -> Result<Vec<DanmakuItem>, String> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut items = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"d" => {
                // 读取 p 属性
                let p = e
                    .attributes()
                    .filter_map(|a| a.ok())
                    .find(|a| a.key.as_ref() == b"p")
                    .and_then(|a| a.unescape_value().ok())
                    .map(|v| v.into_owned());

                let text = reader.read_text(e.name()).map_err(|e| e.to_string())?;

                if let Some(p) = p {
                    let parts: Vec<&str> = p.split(',').collect();
                    if parts.len() >= 4 {
                        let time = parts[0].parse::<f64>().unwrap_or(0.0);
                        let mode = parts[1].parse::<i32>().unwrap_or(1);
                        let fontsize = parts[2].parse::<i32>().unwrap_or(25);
                        let color = parts[3].parse::<u32>().unwrap_or(16777215);
                        items.push(DanmakuItem {
                            time,
                            mode,
                            fontsize,
                            color,
                            text: text.to_string(),
                        });
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML解析错误: {e}")),
            _ => {}
        }
        buf.clear();
    }

    Ok(items)
}

// 获取字幕列表
pub async fn fetch_subtitle_list(
    bvid: &str,
    cid: i64,
    cookie: &str,
) -> Result<Vec<SubtitleMeta>, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .map_err(|e| e.to_string())?;

    let (img_key, sub_key) = fetch_wbi_keys(&client).await?;
    let mixin_key = get_mixin_key(&img_key, &sub_key);

    let mut params = std::collections::BTreeMap::new();
    params.insert("bvid".into(), bvid.to_string());
    params.insert("cid".into(), cid.to_string());
    sign_params(&mut params, &mixin_key);

    let mut req = client
        .get("https://api.bilibili.com/x/player/wbi/v2")
        .query(&params)
        .header("Referer", "https://www.bilibili.com/");

    if !cookie.is_empty() {
        req = req.header("Cookie", cookie);
    }

    let resp: serde_json::Value = req
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let subs = resp["data"]["subtitle"]["subtitles"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    Ok(subs
        .into_iter()
        .filter_map(|s| {
            Some(SubtitleMeta {
                lan: s["lan"].as_str()?.to_string(),
                lan_doc: s["lan_doc"].as_str().unwrap_or("").to_string(),
                url: s["subtitle_url"].as_str()?.to_string(),
            })
        })
        .collect())
}

#[derive(serde::Serialize, Clone)]
pub struct SubtitleMeta {
    pub lan: String,
    pub lan_doc: String,
    pub url: String, // 需要补全 https: 前缀
}

// 下载字幕 JSON 并解析
pub async fn fetch_subtitle_content(url: &str) -> Result<Vec<SubtitleCue>, String> {
    let full_url = if url.starts_with("//") {
        format!("https:{url}")
    } else {
        url.to_string()
    };
    let resp = reqwest::Client::builder()
        .user_agent(BILI_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?
        .get(&full_url)
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("字幕请求失败 HTTP {status}: {full_url}"));
    }
    let text = resp.text().await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|e| format!("字幕 JSON 解析失败: {e}"))?;
    let body = json["body"].as_array().cloned().unwrap_or_default();

    Ok(body
        .into_iter()
        .filter_map(|b| {
            Some(SubtitleCue {
                from: b["from"].as_f64()?,
                to: b["to"].as_f64()?,
                content: b["content"].as_str()?.to_string(),
            })
        })
        .collect())
}

#[derive(serde::Serialize, Clone)]
pub struct SubtitleCue {
    pub from: f64,
    pub to: f64,
    pub content: String,
}
