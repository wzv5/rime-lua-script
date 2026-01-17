use std::{collections::HashMap, sync::Arc};

use mlua::prelude::*;
use serde::Deserialize;

use crate::error::AnyError;

pub enum PostProcessing {
    // 原始顺序，仅去重
    None,
    // 按字符串长度排序
    SortByLength,
    // 截断至输入的拼音的长度，并按出现次数排序
    Truncate,
}

pub struct Suggest {
    client: reqwest::Client,
    runtime: tokio::runtime::Runtime,
    providers: Vec<String>,
    post_processing: PostProcessing,
}

impl Suggest {
    pub fn new() -> LuaResult<Self> {
        let client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(1))
            .build()
            .map_err(|e| e.into_lua_err())?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(Self {
            client,
            runtime,
            providers: vec![],
            post_processing: PostProcessing::None,
        })
    }

    pub fn set_providers(&mut self, providers: Vec<String>) {
        self.providers = providers;
    }

    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_millis(timeout_ms))
            .build()
            .unwrap();
    }

    pub fn set_post_processing(&mut self, post_processing: PostProcessing) {
        self.post_processing = post_processing;
    }

    pub fn call(&self, pinyin: Vec<String>) -> Vec<String> {
        if pinyin.is_empty() || self.providers.is_empty() {
            return vec![];
        }
        let pinyin_len = pinyin.len();
        let data = self.runtime.block_on(self.call_async(Arc::new(pinyin)));
        let mut map: HashMap<String, i32> = HashMap::new();
        for v in data {
            for (index, value) in v.into_iter().enumerate() {
                let score = (100 - 5 * index) as i32;
                map.entry(value)
                    .and_modify(|v| {
                        if *v < score {
                            *v = score;
                        }
                        *v += 5
                    })
                    .or_insert(score);
            }
        }
        let result = match self.post_processing {
            PostProcessing::None => {
                let mut kv: Vec<_> = map.into_iter().collect();
                kv.sort_by_key(|kv| -kv.1);
                kv.into_iter().map(|(i, _)| i).collect()
            }
            PostProcessing::SortByLength => {
                let mut result: Vec<_> = map.keys().cloned().collect();
                result.sort_by_key(|i| i.chars().count());
                result
            }
            PostProcessing::Truncate => {
                let mut map2: HashMap<String, i32> = HashMap::new();
                for (k, mut v) in map.into_iter() {
                    if k.chars().count() == pinyin_len {
                        v = 105;
                    }
                    let k: String = k.chars().take(pinyin_len).collect();
                    map2.entry(k)
                        .and_modify(|score| {
                            if *score < v {
                                *score = v;
                            }
                            *score += 2;
                        })
                        .or_insert(v);
                }
                let mut kv: Vec<_> = map2.into_iter().collect();
                kv.sort_by_key(|kv| -kv.1);
                kv.into_iter().map(|(i, _)| i).collect()
            }
        };
        result
    }

    async fn call_async(&self, pinyin: Arc<Vec<String>>) -> Vec<Vec<String>> {
        let mut tasks = Vec::with_capacity(self.providers.len());
        for i in &self.providers {
            let client = self.client.clone();
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
        let mut outputs = Vec::with_capacity(self.providers.len());
        for i in tasks {
            if let Ok(i) = i.await {
                if let Ok(i) = i
                    && !i.is_empty()
                {
                    outputs.push(i);
                }
            }
        }
        outputs
    }
}

impl LuaUserData for Suggest {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("call", |_, this, pinyin| Ok(this.call(pinyin)));
    }

    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_set("providers", |_, this, providers| {
            this.set_providers(providers);
            Ok(())
        });
        fields.add_field_method_set("timeout", |_, this, timeout| {
            this.set_timeout(timeout);
            Ok(())
        });
        fields.add_field_method_set("post_processing", |_, this, post_processing: String| {
            let a = match post_processing.as_str() {
                "sort_by_length" => PostProcessing::SortByLength,
                "truncate" => PostProcessing::Truncate,
                _ => PostProcessing::None,
            };
            this.set_post_processing(a);
            Ok(())
        });
    }
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
