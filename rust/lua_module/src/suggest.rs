use std::sync::Arc;

use serde::Deserialize;

use crate::error::AnyError;

pub fn suggest(pinyin: Vec<String>, providers: Vec<String>) -> Vec<String> {
    if pinyin.is_empty() || providers.is_empty() {
        return vec![];
    }
    // 减少多线程创建开销，仅使用单线程模式也够用了
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { suggest_async(pinyin, providers).await.unwrap_or_default() })
}

async fn suggest_async(
    pinyin: Vec<String>,
    providers: Vec<String>,
) -> Result<Vec<String>, AnyError> {
    let client = Arc::new(
        reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(1))
            .build()?,
    );
    let pinyin = Arc::new(pinyin);
    let mut tasks = Vec::with_capacity(providers.len());
    for i in &providers {
        let client = client.clone();
        let pinyin = pinyin.clone();
        match i.as_str() {
            "baidu" => tasks.push(tokio::spawn(async move {
                baidu(&client, pinyin.as_slice()).await
            })),
            "bilibili" => tasks.push(tokio::spawn(async move {
                bilibili(&client, pinyin.as_slice()).await
            })),
            "bing" => tasks.push(tokio::spawn(async move {
                bing(&client, pinyin.as_slice()).await
            })),
            "taobao" => tasks.push(tokio::spawn(async move {
                taobao(&client, pinyin.as_slice()).await
            })),
            _ => {}
        }
    }
    let mut outputs = Vec::with_capacity(providers.len());
    for i in tasks {
        if let Ok(i) = i.await {
            if let Ok(i) = i
                && !i.is_empty()
            {
                outputs.push(i);
            }
        }
    }
    let max_len = outputs.iter().map(|i| i.len()).max().unwrap_or(0);
    let mut result = Vec::with_capacity(max_len * outputs.len());
    for i in 0..max_len {
        for v in &outputs {
            if let Some(item) = v.get(i) {
                result.push(item.clone());
            }
        }
    }
    Ok(result)
}

async fn baidu(client: &reqwest::Client, pinyin: &[String]) -> Result<Vec<String>, AnyError> {
    let result = client
        .get("https://www.baidu.com/sugrec?pre=1&p=3&ie=utf-8&json=1&prod=pc&from=pc_web")
        .query(&[("wd", pinyin.join(" "))])
        .send()
        .await?
        .json::<BaiduJson>()
        .await?
        .g
        .into_iter()
        // 只选取 type == "sug"，其他结果疑似广告
        .filter_map(|i| if i.r#type == "sug" { Some(i.q) } else { None })
        .collect();
    Ok(result)
}

async fn bilibili(client: &reqwest::Client, pinyin: &[String]) -> Result<Vec<String>, AnyError> {
    let result = client
        .get("https://api.bilibili.com/x/web-interface/suggest")
        .query(&[("term", pinyin.join(" "))])
        .send()
        .await?
        .json::<BilibiliJson>()
        .await?
        .data
        .result
        .tag
        .into_iter()
        .map(|i| i.value)
        .collect();
    Ok(result)
}

async fn bing(client: &reqwest::Client, pinyin: &[String]) -> Result<Vec<String>, AnyError> {
    let result = client
        .get("https://cn.bing.com/AS/Suggestions?cvid=1&csr=1")
        .query(&[("qry", pinyin.join(" "))])
        .send()
        .await?
        .json::<BingJson>()
        .await?
        .s
        .into_iter()
        // 使用特殊字符作为分隔符，两分隔符之间的内容在网页中加粗显示
        .map(|i| i.q.replace("\u{e000}", "").replace("\u{e001}", ""))
        .collect();
    Ok(result)
}

async fn taobao(client: &reqwest::Client, pinyin: &[String]) -> Result<Vec<String>, AnyError> {
    let result = client
        .get("https://suggest.taobao.com/sug?code=utf-8")
        // 不能加空格分割，不然没结果
        .query(&[("q", pinyin.join(""))])
        .send()
        .await?
        .json::<TaobaoJson>()
        .await?
        .result
        .into_iter()
        .filter_map(|i| {
            if let Some(s) = i.into_iter().next() {
                Some(s)
            } else {
                None
            }
        })
        .collect();
    Ok(result)
}

#[derive(Deserialize)]
struct BaiduJson {
    g: Vec<BaiduJsonG>,
}

#[derive(Deserialize)]
struct BaiduJsonG {
    r#type: String,
    q: String,
}

#[derive(Deserialize)]
struct BilibiliJson {
    data: BilibiliJsonData,
}

#[derive(Deserialize)]
struct BilibiliJsonData {
    result: BilibiliJsonResult,
}

#[derive(Deserialize)]
struct BilibiliJsonResult {
    tag: Vec<BilibiliJsonTag>,
}

#[derive(Deserialize)]
struct BilibiliJsonTag {
    value: String,
}

#[derive(Deserialize)]
struct BingJson {
    s: Vec<BingJsonS>,
}

#[derive(Deserialize)]
struct BingJsonS {
    q: String,
}

#[derive(Deserialize)]
struct TaobaoJson {
    result: Vec<Vec<String>>,
}
