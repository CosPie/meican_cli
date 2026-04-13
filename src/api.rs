use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use reqwest::redirect::Policy;

use crate::error::MeicanError;
use crate::models::*;

const MEICAN_BASE_URL: &str = "https://meican.com/preorder/api/v2.1";
const MEICAN_LOGIN_URL: &str = "https://meican.com/account/directlogin";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub struct MeicanClient {
    client: reqwest::Client,
    cookie: String,
}

impl MeicanClient {
    pub fn new(cookie: &str) -> Self {
        let client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()
            .expect("Failed to build HTTP client");
        Self {
            client,
            cookie: cookie.to_string(),
        }
    }

    fn timestamp() -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string()
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}{}", MEICAN_BASE_URL, path);

        let mut all_query: Vec<(&str, &str)> = vec![];
        let ts = Self::timestamp();
        all_query.push(("noHttpGetCache", &ts));
        all_query.extend_from_slice(query);

        let resp = self
            .client
            .get(&url)
            .query(&all_query)
            .header("Accept", "application/json")
            .header("User-Agent", USER_AGENT)
            .header("Origin", "https://meican.com")
            .header("Referer", "https://meican.com/")
            .header("Cookie", &self.cookie)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() && !status.is_redirection() {
            let text = resp.text().await.unwrap_or_default();
            return Err(MeicanError::ApiError {
                status: status.as_u16(),
                message: text,
            }
            .into());
        }

        let text = resp.text().await?;
        serde_json::from_str(&text).map_err(|e| {
            MeicanError::InvalidResponse(format!("{e}: {}", &text[..text.len().min(200)])).into()
        })
    }

    async fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        form: &HashMap<&str, &str>,
    ) -> Result<T> {
        let url = format!("{}{}", MEICAN_BASE_URL, path);
        let ts = Self::timestamp();

        let resp = self
            .client
            .post(&url)
            .query(&[("noHttpGetCache", ts.as_str())])
            .header("Accept", "application/json")
            .header("User-Agent", USER_AGENT)
            .header("Origin", "https://meican.com")
            .header("Referer", "https://meican.com/")
            .header("Cookie", &self.cookie)
            .form(form)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() && !status.is_redirection() {
            let text = resp.text().await.unwrap_or_default();
            return Err(MeicanError::ApiError {
                status: status.as_u16(),
                message: text,
            }
            .into());
        }

        let text = resp.text().await?;
        serde_json::from_str(&text).map_err(|e| {
            MeicanError::InvalidResponse(format!("{e}: {}", &text[..text.len().min(200)])).into()
        })
    }

    // ========================================================================
    // Login (static, doesn't need stored cookie)
    // ========================================================================

    pub async fn login(username: &str, password: &str) -> Result<String> {
        let client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()?;

        let resp = client
            .post(MEICAN_LOGIN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("User-Agent", USER_AGENT)
            .header("Accept", "application/json, text/plain, */*")
            .header("Origin", "https://meican.com")
            .header("Referer", "https://meican.com/")
            .form(&[
                ("username", username),
                ("password", password),
                ("loginType", "username"),
                ("remember", "true"),
            ])
            .send()
            .await?;

        let response_text = resp
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|v| {
                v.to_str()
                    .unwrap_or("")
                    .split(';')
                    .next()
                    .unwrap_or("")
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        let body = resp.text().await.unwrap_or_default();

        if body.contains("用户名或密码错误") || body.contains("login fail") {
            return Err(MeicanError::LoginFailed("Invalid username or password".into()).into());
        }

        let cookies = response_text.join("; ");

        if cookies.is_empty() {
            return Err(MeicanError::LoginFailed("No cookies received".into()).into());
        }

        let has_play_session =
            cookies.contains("PLAY_SESSION=") && !cookies.contains("PLAY_SESSION=;");
        let has_error = cookies.contains("PLAY_ERRORS=") && !cookies.contains("PLAY_ERRORS=;");

        // Check for encoded error in PLAY_FLASH
        if let Some(flash_start) = cookies.find("PLAY_FLASH=\"") {
            let flash_content = &cookies[flash_start..];
            if let Some(end) = flash_content[12..].find('"') {
                let decoded = urlencoding::decode(&flash_content[12..12 + end]).unwrap_or_default();
                if decoded.contains("error=") {
                    return Err(
                        MeicanError::LoginFailed("Invalid username or password".into()).into(),
                    );
                }
            }
        }

        if !has_play_session {
            return Err(MeicanError::LoginFailed("No valid session received".into()).into());
        }
        if has_error {
            return Err(MeicanError::LoginFailed("Login returned an error".into()).into());
        }

        Ok(cookies)
    }

    // ========================================================================
    // Calendar
    // ========================================================================

    pub async fn get_calendar(
        &self,
        begin_date: &str,
        end_date: &str,
        with_order_detail: bool,
    ) -> Result<CalendarResponse> {
        self.get(
            "/calendarItems/list",
            &[
                ("beginDate", begin_date),
                ("endDate", end_date),
                (
                    "withOrderDetail",
                    if with_order_detail { "true" } else { "false" },
                ),
            ],
        )
        .await
    }

    // ========================================================================
    // Addresses
    // ========================================================================

    pub async fn get_addresses(&self, namespace: &str) -> Result<AddressResponse> {
        let mut query = vec![];
        if !namespace.is_empty() {
            query.push(("namespace", namespace));
        }
        self.get("/corpaddresses/getmulticorpaddress", &query).await
    }

    // ========================================================================
    // Restaurants
    // ========================================================================

    pub async fn get_restaurants(
        &self,
        tab_unique_id: &str,
        target_time: &str,
    ) -> Result<RestaurantsResponse> {
        self.get(
            "/restaurants/list",
            &[("tabUniqueId", tab_unique_id), ("targetTime", target_time)],
        )
        .await
    }

    // ========================================================================
    // Dishes
    // ========================================================================

    pub async fn get_dishes(
        &self,
        tab_unique_id: &str,
        target_time: &str,
    ) -> Result<DishesResponse> {
        self.get(
            "/recommendations/dishes",
            &[("tabUniqueId", tab_unique_id), ("targetTime", target_time)],
        )
        .await
    }

    // ========================================================================
    // Orders
    // ========================================================================

    pub async fn add_order(
        &self,
        tab_unique_id: &str,
        order_json: &str,
        remarks_json: &str,
        target_time: &str,
        user_address_id: &str,
        corp_address_id: &str,
    ) -> Result<OrderResponse> {
        let mut form = HashMap::new();
        form.insert("tabUniqueId", tab_unique_id);
        form.insert("order", order_json);
        form.insert("remarks", remarks_json);
        form.insert("targetTime", target_time);
        form.insert("userAddressUniqueId", user_address_id);
        form.insert("corpAddressUniqueId", corp_address_id);

        self.post("/orders/add", &form).await
    }

    pub async fn delete_order(&self, unique_id: &str) -> Result<serde_json::Value> {
        let mut form = HashMap::new();
        form.insert("uniqueId", unique_id);
        form.insert("type", "CORP_ORDER");
        form.insert("restoreCart", "false");

        self.post("/orders/delete", &form).await
    }
}
