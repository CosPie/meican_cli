use anyhow::{bail, Result};
use chrono::Local;
use colored::Colorize;

use crate::api::MeicanClient;
use crate::commands::menu::resolve_tab_by_meal;
use crate::config;
use crate::display;
use crate::Meal;

fn build_client() -> Result<MeicanClient> {
    let session = config::load_session()?;
    Ok(MeicanClient::new(&session.cookies))
}

fn meal_time_suffix(title: &str) -> &'static str {
    let t = title.to_lowercase();
    if t.contains("早餐") || t.contains("breakfast") {
        "07:00"
    } else if t.contains("晚餐") || t.contains("dinner") {
        "12:00"
    } else {
        "09:00"
    }
}

struct OrderContext {
    tab_unique_id: String,
    target_time_str: String,
    namespace: String,
}

async fn resolve_order_context(
    client: &MeicanClient,
    meal: Option<&Meal>,
    tab: Option<&str>,
) -> Result<OrderContext> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let resp = client.get_calendar(&today, &today, true).await?;

    // Find the target tab
    let target_tab_id = if let Some(t) = tab {
        t.to_string()
    } else {
        let resolved = resolve_tab_by_meal(client, meal).await?;
        resolved.tab_unique_id
    };

    for date_item in &resp.date_list {
        for cal_item in &date_item.calendar_item_list {
            let tab_id = cal_item
                .user_tab
                .as_ref()
                .map(|t| t.unique_id.as_str());

            if tab_id == Some(&target_tab_id) {
                let namespace = cal_item
                    .user_tab
                    .as_ref()
                    .and_then(|t| t.corp.as_ref())
                    .and_then(|c| c.namespace.clone())
                    .or_else(|| cal_item.corp.as_ref().and_then(|c| c.namespace.clone()))
                    .or_else(|| {
                        cal_item
                            .corp_order_user
                            .as_ref()
                            .and_then(|c| c.corp.as_ref())
                            .and_then(|c| c.namespace.clone())
                    })
                    .ok_or_else(|| {
                        anyhow::anyhow!("No namespace found for tab '{}'", target_tab_id)
                    })?;

                let time_suffix = meal_time_suffix(&cal_item.title);
                let target_time_str = format!("{} {}", date_item.date, time_suffix);

                return Ok(OrderContext {
                    tab_unique_id: target_tab_id,
                    target_time_str,
                    namespace,
                });
            }
        }
    }

    bail!("Tab ID '{}' not found in today's calendar", target_tab_id)
}

async fn resolve_address(client: &MeicanClient, namespace: &str) -> Result<String> {
    let addr_resp = client.get_addresses(namespace).await?;
    addr_resp
        .data
        .as_ref()
        .and_then(|d| {
            d.recent_list
                .as_ref()
                .and_then(|list| list.first())
                .and_then(|a| a.unique_id.clone())
                .or_else(|| {
                    d.address_list
                        .as_ref()
                        .and_then(|list| list.first())
                        .and_then(|w| w.final_value.as_ref())
                        .and_then(|a| a.unique_id.clone())
                })
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No delivery address found for namespace '{}'. Please set an address via the web interface.",
                namespace
            )
        })
}

pub async fn add_order(
    meal: Option<Meal>,
    tab: Option<&str>,
    dish_id: &str,
    table: bool,
) -> Result<()> {
    let client = build_client()?;
    let ctx = resolve_order_context(&client, meal.as_ref(), tab).await?;
    let address_id = resolve_address(&client, &ctx.namespace).await?;

    let order_json = format!(r#"[{{"count":1,"dishId":{}}}]"#, dish_id);
    let remarks_json = format!(r#"[{{"dishId":"{}","remark":""}}]"#, dish_id);

    let resp = client
        .add_order(
            &ctx.tab_unique_id,
            &order_json,
            &remarks_json,
            &ctx.target_time_str,
            &address_id,
            &address_id,
        )
        .await?;

    display::print_order_result(&resp, table);
    Ok(())
}

pub async fn cancel_order(meal: Option<Meal>, id: Option<&str>) -> Result<()> {
    let client = build_client()?;

    let order_id = if let Some(id) = id {
        id.to_string()
    } else {
        // Find today's order for the given meal
        let today = Local::now().format("%Y-%m-%d").to_string();
        let resp = client.get_calendar(&today, &today, true).await?;

        let mut found_id = None;
        for date_item in &resp.date_list {
            for cal_item in &date_item.calendar_item_list {
                if cal_item.status != "ORDER" {
                    continue;
                }

                let should_match = match &meal {
                    Some(m) => {
                        let t = cal_item.title.to_lowercase();
                        match m {
                            Meal::Breakfast => {
                                t.contains("早餐") || t.contains("breakfast")
                            }
                            Meal::Lunch => t.contains("午餐") || t.contains("lunch"),
                            Meal::Dinner => t.contains("晚餐") || t.contains("dinner"),
                        }
                    }
                    None => true, // No meal specified, cancel first found
                };

                if should_match {
                    if let Some(corp) = &cal_item.corp_order_user {
                        found_id = Some(corp.unique_id.clone());
                        break;
                    }
                }
            }
            if found_id.is_some() {
                break;
            }
        }

        found_id.ok_or_else(|| match &meal {
            Some(m) => anyhow::anyhow!("No {:?} order found to cancel today", m),
            None => anyhow::anyhow!("No order found to cancel today"),
        })?
    };

    let resp = client.delete_order(&order_id).await?;
    let detail = serde_json::to_string_pretty(&resp).unwrap_or_default();
    println!("{}", "Order cancelled.".green().bold());
    println!("{}", detail);
    Ok(())
}
