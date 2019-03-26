use std::error::Error;
use std::fmt;
use std::env;

use chrono::prelude::*;
use hmac::{Hmac, Mac};
use reqwest::{header, Client};
use serde_json::json;
use sha2::{Sha256, Digest};
use url::Url;
use uuid::Uuid;

const TIMESTAMP_FORMAT: &str = "%Y%m%dT%H:%M:%S+0000";

type HmacSha256 = Hmac<Sha256>;

fn signing_key(client_secret: &str, now: DateTime<Utc>) -> String {
    let timestamp = now.format(TIMESTAMP_FORMAT).to_string();
    let mut mac = HmacSha256::new_varkey(client_secret.as_bytes()).unwrap();
    mac.input(timestamp.as_bytes());
    base64::encode(&mac.result().code())
}

fn signature(signing_key: &str, data_to_sign: &str) -> String {
    let mut mac = HmacSha256::new_varkey(signing_key.as_bytes()).unwrap();
    mac.input(data_to_sign.as_bytes());
    base64::encode(&mac.result().code())
}

fn body_hash(body: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.input(body);
    base64::encode(&hasher.result())
}

#[derive(Debug, Copy, Clone)]
pub enum HttpMethod {
    Get,
    Post,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpMethod::Get => f.write_str("GET"),
            HttpMethod::Post => f.write_str("POST"),
        }
    }
}

pub fn authorization_header(
    method: HttpMethod,
    url: &Url,
    body: Option<&[u8]>,
    access_token: &str,
    client_token: &str,
    client_secret: &str,
) -> String
{
    let now_timestamp = Utc::now();
    let signing_key = signing_key(client_secret, now_timestamp);

    let nonce = Uuid::new_v4().to_hyphenated();

    let mut auth_data =
        format!("EG1-HMAC-SHA256 client_token={};access_token={};timestamp={};nonce={};",
            client_token,
            access_token,
            now_timestamp.format(TIMESTAMP_FORMAT).to_string(),
            nonce,
        );

    let data_to_sign = {
        let method = method.to_string();
        let scheme = url.scheme();
        let host = url.host_str().expect("HOST not found");
        let relurl = url.path();
        let mut data_to_sign = format!("{}\t{}\t{}\t{}\t\t", method, scheme, host, relurl);
        if let Some(body) = body {
            let body_hash = body_hash(body);
            data_to_sign.push_str(&body_hash);
        }
        data_to_sign.push('\t');
        data_to_sign.push_str(&auth_data);
        data_to_sign
    };

    let signature = signature(&signing_key, &data_to_sign);
    auth_data.push_str(&format!("signature={}", signature));

    auth_data
}

pub fn purge_tag(tags: Vec<String>) -> Result<(), Box<Error>> {
    let access_token = env::var("AKAMAI_ACCESS_TOKEN")?;
    // println!("AKAMAI_ACCESS_TOKEN");
    let client_token = env::var("AKAMAI_CLIENT_TOKEN")?;
    // println!("AKAMAI_CLIENT_TOKEN");
    let client_secret = env::var("AKAMAI_CLIENT_SECRET")?;
    // println!("AKAMAI_ClIENT_SECRET");
    let url = env::var("AKAMAI_URL")?;
    // println!("AKAMAI_URL");

    let body = json!(
        {
            "objects": tags
        }
    ).to_string();

    let client = Client::builder().build()?;

    let url_string = format!("{}/ccu/v3/invalidate/tag/{}", url, "production");
    let url = Url::parse(&url_string)?;


    let auth_data =
        authorization_header(
            HttpMethod::Post,
            &url,
            Some(body.as_bytes()),
            &access_token,
            &client_token,
            &client_secret
        );

    let req = client
        .post(url)
        .header(header::AUTHORIZATION, auth_data)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .build()?;

    let _ = client.execute(req)?;

    Ok(())
}
