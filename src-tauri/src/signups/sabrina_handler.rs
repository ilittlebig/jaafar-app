use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use ua_generator::ua::spoof_ua;

use crate::services::captcha_service;
use crate::services::files_service;

#[derive(Debug, Deserialize)]
struct Account {
    email: String,
    firstname: String,
    lastname: String,
    phone: String,
    address1: String,
    address2: String,
    city: String,
    postcode: String,
    country: String,
}

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: Value,
}

const CAPTCHA_WEBSITE_URL: &str = "https://embed.laylo.com";
const CAPTCHA_WEBSITE_KEY: &str = "6LfaRWApAAAAAPvWsG2tsIhBCLEdXyz_EUQtQily";
const CAPTCHA_TASK_TYPE: &str = "ReCaptchaV3TaskProxyLess";
const CAPTCHA_PAGE_ACTION: &str = "create_customer";

const REQUEST_URL: &str = "https://laylo.com/api/graphql";

const DROP_DATE: i64 = 1733386800000;
const FINGERPRINT_ID: &str = "3542ca6347494934b9e7be13fa3922e7";
const OPT_IN: bool = false;
const REFERRER: &str = "https://store.sabrinacarpenter.com/pages/tour";

const QUERY: &str = r#"
    mutation (
      $productId: ID!
      $userId: String
      $email: String
      $dropDate: Float
      $fingerprintId: String
      $phoneNumber: String
      $optIn: Boolean
      $referrer: String
      $captcha: String
      $instagramId: String
    ) {
      purchaseProduct(
        productId: $productId
        userId: $userId
        email: $email
        phoneNumber: $phoneNumber
        dropDate: $dropDate
        fingerprintId: $fingerprintId
        optIn: $optIn
        referrer: $referrer
        captcha: $captcha
        instagramId: $instagramId
      )
    }
"#;

pub async fn run(app_handle: tauri::AppHandle, proxy_group: String, mode: String, product_id: &str) -> Result<(), String> {
    let accounts_path = files_service::resolve_path(&app_handle, "accounts.json")?;
    let proxies_path = files_service::resolve_path(&app_handle, &format!("proxies/{}.json", proxy_group))?;
    let settings_path = files_service::resolve_path(&app_handle, "settings.json")?;

    let accounts: Vec<Account> = files_service::read_json_file(accounts_path)?;

    let client_key = "CAP-50C1E71AF959204A1440C4E490AA036E".to_string();
    let captcha_solution = captcha_service::solve_captcha(
        client_key,
        CAPTCHA_WEBSITE_URL,
        CAPTCHA_WEBSITE_KEY,
        CAPTCHA_TASK_TYPE,
        Some(CAPTCHA_PAGE_ACTION),
        None,
    ).await.map_err(|e| format!("Captcha solving failed: {}", e))?;

    let variables = serde_json::json!({
        "dropDate": DROP_DATE,
        "fingerprintId": FINGERPRINT_ID,
        "productId": product_id,
        "optIn": OPT_IN,
        "email": "elias@jamee.se",
        "referrer": REFERRER,
        "captcha": captcha_solution,
    });

    let graphql_request = GraphQLRequest {
        query: QUERY.to_string(),
        variables,
    };

    let client = Client::new();
    let user_agent = spoof_ua();

    let response = client
        .post(REQUEST_URL)
        .header("User-Agent", user_agent)
        .header("Referer", CAPTCHA_WEBSITE_URL)
        .header("Origin", CAPTCHA_WEBSITE_URL)
        .json(&graphql_request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    println!("Response: {}", response_text);

    Ok(())
}