use reqwest::Client;
use scraper::{Html, Selector};
use serde::Serialize;

const WEB_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36";

#[derive(Debug, Serialize)]
pub struct WebPage {
    pub title: String,
    pub author: Option<String>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub content_html: String,
    pub publish_time: Option<String>,
    pub site_name: String,
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn is_github_repo(url: &str) -> Option<(String, String)> {
    let parsed = url::Url::parse(url).ok()?;
    if parsed.host_str()? != "github.com" && parsed.host_str()? != "www.github.com" {
        return None;
    }
    let parts: Vec<&str> = parsed.path_segments()?.filter(|p| !p.is_empty()).collect();
    if parts.len() < 2 {
        return None;
    }
    Some((parts[0].to_string(), parts[1].to_string()))
}

pub async fn fetch_page(url: &str) -> Result<WebPage, String> {
    if let Some((owner, repo)) = is_github_repo(url) {
        if let Ok(page) = fetch_github_page(&owner, &repo).await {
            return Ok(page);
        }
    }
    fetch_html_page(url).await
}

async fn fetch_github_page(owner: &str, repo: &str) -> Result<WebPage, String> {
    let client = Client::builder()
        .user_agent("Collectrix")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = client
        .get(format!("https://api.github.com/repos/{owner}/{repo}"))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    if !json["full_name"].is_string() {
        return Err("GitHub 接口未返回仓库信息".into());
    }

    let full_name = json["full_name"].as_str().unwrap_or(repo).to_string();
    let description = json["description"].as_str().map(String::from);
    let author = json["owner"]["login"].as_str().map(String::from);
    let cover_url = json["owner"]["avatar_url"].as_str().map(String::from);
    let language = json["language"].as_str().unwrap_or("—");
    let stars = json["stargazers_count"].as_i64().unwrap_or(0);
    let forks = json["forks_count"].as_i64().unwrap_or(0);
    let homepage = json["homepage"].as_str().filter(|value| !value.is_empty());
    let updated_at = json["updated_at"].as_str().map(String::from);

    let mut html = String::new();
    if let Some(text) = &description {
        html.push_str(&format!("<p>{}</p>", escape_html(text)));
    }
    html.push_str("<ul>");
    html.push_str(&format!("<li>Language: {}</li>", escape_html(language)));
    html.push_str(&format!("<li>Stars: {stars}</li>"));
    html.push_str(&format!("<li>Forks: {forks}</li>"));
    if let Some(homepage) = homepage {
        html.push_str(&format!(
            "<li>Homepage: <a href=\"{}\">{}</a></li>",
            escape_html(homepage),
            escape_html(homepage)
        ));
    }
    html.push_str("</ul>");

    Ok(WebPage {
        title: full_name,
        author,
        cover_url,
        description,
        content_html: html,
        publish_time: updated_at,
        site_name: "GitHub".into(),
    })
}

fn meta_content(doc: &Html, attr: &str, name: &str) -> Option<String> {
    let selector = Selector::parse(&format!("meta[{attr}='{name}']")).ok()?;
    doc.select(&selector)
        .next()
        .and_then(|element| element.value().attr("content"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
}

fn first_text(doc: &Html, selector: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    doc.select(&selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn fetch_html_page(url: &str) -> Result<WebPage, String> {
    let client = Client::builder()
        .user_agent(WEB_USER_AGENT)
        .timeout(std::time::Duration::from_secs(25))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(url)
        .header("Accept", "text/html,application/xhtml+xml")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("网页请求失败 HTTP {}", resp.status()));
    }
    let html = resp.text().await.map_err(|e| e.to_string())?;
    let doc = Html::parse_document(&html);

    let title = meta_content(&doc, "property", "og:title")
        .or_else(|| meta_content(&doc, "name", "twitter:title"))
        .or_else(|| first_text(&doc, "title"))
        .unwrap_or_else(|| url.to_string());
    let description = meta_content(&doc, "property", "og:description")
        .or_else(|| meta_content(&doc, "name", "description"))
        .or_else(|| meta_content(&doc, "name", "twitter:description"));
    let cover_url = meta_content(&doc, "property", "og:image")
        .or_else(|| meta_content(&doc, "name", "twitter:image"));
    let author = meta_content(&doc, "name", "author")
        .or_else(|| meta_content(&doc, "property", "article:author"));
    let publish_time = meta_content(&doc, "property", "article:published_time").or_else(|| {
        let selector = Selector::parse("time[datetime]").ok()?;
        doc.select(&selector)
            .next()
            .and_then(|element| element.value().attr("datetime"))
            .map(String::from)
    });
    let site_name = meta_content(&doc, "property", "og:site_name")
        .or_else(|| url::Url::parse(url).ok().and_then(|u| u.host_str().map(String::from)))
        .unwrap_or_default();

    let mut content_html = String::new();
    for selector in [
        "article",
        "main",
        "[role='main']",
        ".article-content",
        ".post-content",
        ".markdown-body",
        "#content",
        ".content",
        "body",
    ] {
        let Ok(selector) = Selector::parse(selector) else {
            continue;
        };
        if let Some(element) = doc.select(&selector).next() {
            if element.text().collect::<String>().trim().len() >= 80 {
                content_html = element.html();
                break;
            }
        }
    }
    if content_html.is_empty() {
        content_html = description
            .as_ref()
            .map(|text| format!("<p>{}</p>", escape_html(text)))
            .unwrap_or_else(|| "<p>未能提取正文内容，请打开原网页查看。</p>".into());
    }
    if content_html.len() > 500_000 {
        let mut end = 500_000;
        while end > 0 && !content_html.is_char_boundary(end) {
            end -= 1;
        }
        content_html.truncate(end);
    }

    Ok(WebPage {
        title,
        author,
        cover_url,
        description,
        content_html,
        publish_time,
        site_name,
    })
}
