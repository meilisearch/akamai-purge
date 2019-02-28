use reqwest::{header, Client};
use url::Url;

use akamai::{authorization_header, HttpMethod};

fn main() {
    let body = r#"
        {
            "objects": [
                "catalog"
            ]
        }
    "#;

    dbg!(&body);

    let access_token = env!("ACCESS_TOKEN");
    let client_token = env!("CLIENT_TOKEN");
    let client_secret = env!("CLIENT_SECRET");

    let client = Client::builder().build().unwrap();

    let base = "https://akab-v6etn6gnkjrsrkuu-ivl4pk6pkkr3yfox.purge.akamaiapis.net";
    let url_string = format!("{}/ccu/v3/invalidate/tag/{}", base, "production");
    let url = Url::parse(&url_string).unwrap();

    let auth_data =
        authorization_header(
            HttpMethod::Post,
            &url,
            Some(body.as_bytes()),
            access_token,
            client_token,
            client_secret
        );

    let req = client
        .post(url)
        .header(header::AUTHORIZATION, auth_data)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .build()
        .unwrap();

    dbg!(&req);

    let mut res = client.execute(req).unwrap();

    println!("{:#?}", res);
    println!("{}", res.text().unwrap());
}
