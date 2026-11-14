//! AI 模块：封装与 OpenAI 兼容服务、Ollama 原生 API 的交互。
//! 不直接操作数据库，只负责 HTTP 请求与 JSON 解析。

use crate::settings::AppSettings;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

// ============ 数据结构 ============

#[derive(Debug, Serialize, Deserialize)]
pub struct AiVideoInput {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub author: String,
    pub cover_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestion {
    pub video_id: i64,
    pub title: String,
    pub cover_url: Option<String>,
    pub suggested_category: String,
    /// 用户选中已有分类时填此字段，apply 时直接采用该 id，忽略 parent_category_id
    pub suggested_category_id: Option<i64>,
    /// 用户指定父分类时填此字段（仅 suggested_category_id 为 None 时生效）
    pub parent_category_id: Option<i64>,
    /// 完整分类路径（从根到叶），用于一次创建多级分类。
    pub category_path: Option<Vec<String>>,
    pub suggested_tags: Vec<String>,
    pub summary: String,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawSuggestion {
    pub id: i64,
    pub category: String,
    pub tags: Vec<String>,
    pub summary: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

fn default_confidence() -> f64 {
    0.7
}

// ============ 通用工具 ============

fn strip_code_fence(s: &str) -> &str {
    let t = s.trim();
    if t.starts_with("```") {
        let start = t.find('\n').map(|i| i + 1).unwrap_or(3);
        let end = t.rfind("```").unwrap_or(t.len());
        return &t[start..end];
    }
    t
}

fn is_reasoning_model(model: &str) -> bool {
    let m = model.to_lowercase();
    m.contains("r1") || m.contains("reasoner") || m.contains("thinking")
}

/// 从任意文本里提取最后一个合法 JSON 对象（跳过 <\think>...</\think> 等前缀）
fn extract_json_from_output(s: &str) -> Option<String> {
    let bytes = s.as_bytes();

    // 用 filter_map 避开借用模式问题：b 是 &u8，*b 就是 u8
    let starts: Vec<usize> = bytes
        .iter()
        .enumerate()
        .filter_map(|(i, b)| if *b == b'{' { Some(i) } else { None })
        .collect();

    for &start in starts.iter().rev() {
        if let Some(end) = find_matching_brace(bytes, start) {
            let candidate = &s[start..=end];
            if serde_json::from_str::<serde_json::Value>(candidate).is_ok() {
                return Some(candidate.to_string());
            }
        }
    }
    None
}

/// 从模型输出中提取合法 JSON；兼容 markdown 代码块和推理模型前缀。
pub(crate) fn parse_json_object(raw: &str) -> Option<String> {
    let cleaned = strip_code_fence(raw);
    if serde_json::from_str::<serde_json::Value>(cleaned).is_ok() {
        Some(cleaned.to_string())
    } else {
        extract_json_from_output(raw)
    }
}

fn find_matching_brace(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;

    for (i, &c) in bytes.iter().enumerate().skip(start) {
        if escape {
            escape = false;
            continue;
        }
        if c == b'\\' && in_string {
            escape = true;
            continue;
        }
        if c == b'"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

fn split_text_chunks(text: &str, max_chars: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let hard_end = (start + max_chars).min(chars.len());
        let mut end = hard_end;
        if hard_end < chars.len() {
            // 尽量在段落或句末切分，避免把一句话拆开。
            let window_start = start + max_chars / 2;
            for index in (window_start..hard_end).rev() {
                if matches!(chars[index], '\n' | '。' | '！' | '？' | '.' | '!' | '?') {
                    end = index + 1;
                    break;
                }
            }
        }
        chunks.push(chars[start..end].iter().collect());
        start = end;
    }
    chunks
}

/// 对语音转录稿做加标点、纠错和分段。长文本会分块处理。
pub(crate) async fn polish_text(
    settings: &AppSettings,
    text: &str,
) -> Result<String, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("没有可校对的文字".into());
    }
    let system = "你是专业的中文文字校对助手。请对用户提供的语音转录稿进行：\n\
        1. 添加准确的标点符号；\n\
        2. 修正明显的同音字、错别字和识别错误；\n\
        3. 按语义合理分段；\n\
        4. 保持原意，不扩写、不删减、不添加原文没有的信息。\n\
        只输出校对后的正文，不要输出解释、标题或 markdown 代码块。";
    let mut output = Vec::new();
    for chunk in split_text_chunks(text, 4_000) {
        let raw = chat_completion(settings, system, &chunk).await?;
        let cleaned = strip_code_fence(&raw).trim().to_string();
        if !cleaned.is_empty() {
            output.push(cleaned);
        }
    }
    Ok(output.join("\n\n"))
}

// ============ 入口分发 ============

pub(crate) async fn chat_completion(
    settings: &AppSettings,
    system: &str,
    user: &str,
) -> Result<String, String> {
    let is_ollama = settings.ai_provider == "ollama"
        || settings.openai_base_url.contains("11434")
        || settings.openai_base_url.contains("localhost");

    if is_ollama {
        chat_completion_ollama(settings, system, user).await
    } else {
        chat_completion_openai(settings, system, user).await
    }
}

// ============ Ollama 原生 API ============

async fn chat_completion_ollama(
    settings: &AppSettings,
    system: &str,
    user: &str,
) -> Result<String, String> {
    // 推导原生 base：http://localhost:11434/v1 → http://localhost:11434
    let base = settings
        .openai_base_url
        .trim_end_matches('/')
        .trim_end_matches("/v1")
        .trim_end_matches("/api")
        .to_string();

    let client = Client::builder()
        .timeout(Duration::from_secs(240))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {e}"))?;

    // ---------- 预热 ----------
    eprintln!("[Ollama] 预热模型 {} …", settings.ai_model);
    let preheat_start = Instant::now();

    let warmup_resp = client
        .post(format!("{base}/api/generate"))
        .json(&serde_json::json!({
            "model": settings.ai_model,
            "prompt": "hi",
            "stream": false,
            "keep_alive": "10m",
            "options": { "num_predict": 5 }
        }))
        .send()
        .await;

    match warmup_resp {
        Ok(r) if r.status().is_success() => {
            eprintln!(
                "[Ollama] 预热完成 (耗时 {:.1}s)",
                preheat_start.elapsed().as_secs_f32()
            );
        }
        Ok(r) => {
            let status = r.status();
            let body = r.text().await.unwrap_or_default();
            return Err(format!(
                "Ollama 预热失败 HTTP {status}: {body}\n\n\
                 检查：\n\
                 1. 模型名是否正确：`ollama list` 查看\n\
                 2. 当前填写的模型: {}",
                settings.ai_model
            ));
        }
        Err(e) => {
            return Err(format!(
                "无法连接 Ollama ({base}): {e}\n\n\
                 检查：\n\
                 1. Ollama 是否运行：终端执行 `ollama serve` 或打开桌面版\n\
                 2. 端口是否被其他程序占用\n\
                 3. Base URL 是否正确: {}",
                settings.openai_base_url
            ));
        }
    }

    // ---------- 正式请求 ----------
    let reasoning = is_reasoning_model(&settings.ai_model);
    eprintln!("[Ollama] → POST {base}/api/chat (stream, reasoning={reasoning})");
    let start = Instant::now();

    let mut body = serde_json::json!({
        "model": settings.ai_model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "stream": true,
        "keep_alive": "10m",
        "options": {
            "temperature": 0.3,
            "num_predict": if reasoning { 8192 } else { 4096 }
        }
    });

    // 只有非推理模型才用 format: json
    if !reasoning {
        body["format"] = serde_json::json!("json");
    }

    let mut resp = client
        .post(format!("{base}/api/chat"))
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            format!(
                "Ollama 请求失败（耗时 {:.1}s）: {e}\n\n\
                 可能是模型生成太慢导致超时。\n\
                 如果使用 7B 及以上模型在 CPU 上运行，建议：\n\
                 1. 换更小的模型：`ollama pull qwen2.5:3b`，设置页改为 qwen2.5:3b\n\
                 2. 减少本次分析数量\n\
                 3. 确认 Ollama 有 GPU 加速（NVIDIA/AMD）",
                start.elapsed().as_secs_f32()
            )
        })?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Ollama HTTP {status}: {text}"));
    }

    // 流式读取 NDJSON
    #[derive(Deserialize)]
    struct StreamChunk {
        message: Option<StreamMsg>,
        done: bool,
    }
    #[derive(Deserialize)]
    struct StreamMsg {
        content: String,
    }

    let mut result = String::new();
    let mut buffer = String::new();
    let mut last_report = Instant::now();

    loop {
        let chunk = match resp.chunk().await {
            Ok(Some(c)) => c,
            Ok(None) => break,
            Err(e) => {
                return Err(format!(
                    "读取流失败（已生成 {} 字符，耗时 {:.1}s）: {e}",
                    result.len(),
                    start.elapsed().as_secs_f32()
                ));
            }
        };

        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find('\n') {
            let line: String = buffer.drain(..=pos).collect();
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(c) = serde_json::from_str::<StreamChunk>(line) {
                if let Some(msg) = c.message {
                    result.push_str(&msg.content);
                }
                if c.done {
                    eprintln!(
                        "[Ollama] ✓ 生成完成，共 {} 字符，耗时 {:.1}s",
                        result.len(),
                        start.elapsed().as_secs_f32()
                    );
                    break;
                }
            }

            if last_report.elapsed() > Duration::from_secs(3) {
                eprintln!(
                    "[Ollama] 生成中… {} 字符，已耗时 {:.1}s",
                    result.len(),
                    start.elapsed().as_secs_f32()
                );
                last_report = Instant::now();
            }
        }
    }

    if result.trim().is_empty() {
        return Err("Ollama 返回空内容，可能是 num_predict 耗尽或模型异常".into());
    }

    // 推理模型的输出里混着 <\think>...</\think>，需要提取 JSON
    let final_text = if reasoning {
        eprintln!("[Ollama] 推理输出 {} 字符，正在提取 JSON…", result.len());
        extract_json_from_output(&result).ok_or_else(|| {
            format!(
                "模型输出里没找到合法 JSON。\n\n输出片段（前 500 字符）：\n{}",
                result.chars().take(500).collect::<String>()
            )
        })?
    } else {
        result
    };

    Ok(final_text)
}

// ============ OpenAI 兼容 API ============

async fn chat_completion_openai(
    settings: &AppSettings,
    system: &str,
    user: &str,
) -> Result<String, String> {
    if settings.openai_api_key.is_empty() {
        return Err("请先在设置中填写 API Key".into());
    }

    let url = format!(
        "{}/chat/completions",
        settings.openai_base_url.trim_end_matches('/')
    );

    eprintln!("[AI] → POST {url}");
    eprintln!(
        "[AI]   模型: {}, 请求体大小: {} bytes",
        settings.ai_model,
        system.len() + user.len()
    );

    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {e}"))?;

    let mut body = serde_json::json!({
        "model": settings.ai_model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "temperature": 0.3,
        "response_format": {"type": "json_object"}
    });

    let start = Instant::now();

    let mut resp = client
        .post(&url)
        .bearer_auth(&settings.openai_api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            let elapsed = start.elapsed().as_secs_f32();
            eprintln!("[AI] ✗ 请求失败 (耗时 {elapsed:.1}s): {e}");
            format!(
                "请求 AI 失败（耗时 {elapsed:.1}s）: {e}\n\n\
                 可能原因：\n\
                 - 网络不通或 Base URL 错误: {url}\n\
                 - Ollama 模型首次加载慢，已超时 120s\n\
                 - 云端服务限流或不可用"
            )
        })?;

    let mut status = resp.status();

    // 部分 OpenAI 兼容服务不支持 response_format，去掉后重试一次。
    if matches!(status.as_u16(), 400 | 422) {
        if let Some(object) = body.as_object_mut() {
            object.remove("response_format");
        }
        eprintln!("[AI] 服务不支持 response_format，正在重试…");
        resp = client
            .post(&url)
            .bearer_auth(&settings.openai_api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("请求 AI 失败: {e}"))?;
        status = resp.status();
    }

    let text = resp.text().await.map_err(|e| e.to_string())?;
    let elapsed = start.elapsed().as_secs_f32();

    eprintln!(
        "[AI] ← HTTP {} (耗时 {elapsed:.1}s, 响应 {} bytes)",
        status,
        text.len()
    );

    if !status.is_success() {
        return Err(format!(
            "AI 请求失败 HTTP {status}\n\n响应: {text}\n\n当前配置：\n- Base URL: {}\n- 模型: {}",
            settings.openai_base_url, settings.ai_model
        ));
    }

    #[derive(Deserialize)]
    struct Resp {
        choices: Vec<Choice>,
    }
    #[derive(Deserialize)]
    struct Choice {
        message: Msg,
    }
    #[derive(Deserialize)]
    struct Msg {
        content: String,
    }

    let r: Resp =
        serde_json::from_str(&text).map_err(|e| format!("解析 AI 响应失败: {e}，原文: {text}"))?;

    r.choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| "AI 返回空 choices".to_string())
}

// ============ 分类建议 ============

pub async fn categorize_batch(
    settings: &AppSettings,
    existing_categories: &[String],
    existing_tags: &[String],
    videos: &[AiVideoInput],
) -> Result<Vec<(i64, RawSuggestion)>, String> {
    let system = format!(
        r#"你是视频分类助手，需要为每个视频指定分类、标签和一句话概要。

已有分类（供参考，优先复用）：{cats}
已有标签（供参考，优先复用）：{tags}

规则：
1. category（分类）：必须选一个。如果已有分类中没有一个合适，可以新建一个简洁的分类名（2-6 个字）。
2. tags（标签）：1~4 个。优先复用已有标签；如果没有合适的，可以新建标签（每个标签 2-6 个字）。
3. summary：一句话概括视频内容（不超过 40 字）。
4. confidence：0~1 之间的小数，表示你对这个分类的把握程度。

严格输出 JSON，不要输出任何解释文字、markdown 代码块或思考过程：
{{"items":[{{"id":123,"category":"...","tags":["...","..."],"summary":"...","confidence":0.85}}]}}"#,
        cats = if existing_categories.is_empty() {
            "（暂无）".to_string()
        } else {
            existing_categories.join("、")
        },
        tags = if existing_tags.is_empty() {
            "（暂无）".to_string()
        } else {
            existing_tags.join("、")
        },
    );

    let user = serde_json::to_string(&serde_json::json!({ "videos": videos }))
        .map_err(|e| e.to_string())?;

    let raw = chat_completion(settings, &system, &user).await?;
    let cleaned = parse_json_object(&raw)
        .ok_or_else(|| format!("AI 未返回合法 JSON，原文: {raw}"))?;

    #[derive(Deserialize)]
    struct Out {
        items: Vec<RawSuggestion>,
    }

    let parsed: Out =
        serde_json::from_str(&cleaned).map_err(|e| format!("AI JSON 解析失败: {e}，原文: {raw}"))?;

    Ok(parsed.items.into_iter().map(|i| (i.id, i)).collect())
}

// ============ 相似度筛选 ============

pub async fn similarity_filter(
    settings: &AppSettings,
    reference: &AiVideoInput,
    candidates: &[AiVideoInput],
) -> Result<Vec<(i64, f64)>, String> {
    let system = "你是内容相似度分析器。给定一个基准视频和候选列表，\
        对每个候选输出 0~1 的相似度（内容主题相近越高）。\
        输出 JSON：{\"items\":[{\"id\":123,\"score\":0.82}]}";

    let user = serde_json::to_string(&serde_json::json!({
        "reference": reference,
        "candidates": candidates,
    }))
    .map_err(|e| e.to_string())?;

    let raw = chat_completion(settings, system, &user).await?;
    let cleaned = parse_json_object(&raw)
        .ok_or_else(|| format!("AI 未返回合法 JSON，原文: {raw}"))?;

    #[derive(Deserialize)]
    struct Out {
        items: Vec<Item>,
    }
    #[derive(Deserialize)]
    struct Item {
        id: i64,
        score: f64,
    }

    let parsed: Out = serde_json::from_str(&cleaned)
        .map_err(|e| format!("AI JSON 解析失败: {e}，原文: {raw}"))?;

    Ok(parsed.items.into_iter().map(|i| (i.id, i.score)).collect())
}
