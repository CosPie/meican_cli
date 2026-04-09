use anyhow::Result;
use chrono::Local;

use crate::api::MeicanClient;
use crate::config;
use crate::display;
use crate::models::*;

fn build_client() -> Result<MeicanClient> {
    let session = config::load_session()?;
    Ok(MeicanClient::new(&session.cookies))
}

fn determine_meal_time(title: &str, opening_name: &str) -> String {
    let t = title.to_lowercase();
    let o = opening_name.to_lowercase();
    if t.contains("早餐") || o.contains("早餐") || t.contains("breakfast") {
        "BREAKFAST".into()
    } else if t.contains("午餐") || o.contains("午餐") || t.contains("lunch") {
        "LUNCH".into()
    } else {
        "DINNER".into()
    }
}

pub async fn history(days: u32, table: bool) -> Result<()> {
    let client = build_client()?;

    let today = Local::now().date_naive();
    let begin = today - chrono::Duration::days(days as i64);
    let mut all_orders: Vec<HistoricalOrder> = Vec::new();

    // Fetch in 15-day chunks to avoid API limits
    let chunk_days = 15i64;
    let mut current_start = begin;

    while current_start < today {
        let current_end_candidate = current_start + chrono::Duration::days(chunk_days - 1);
        let current_end = if current_end_candidate > today {
            today
        } else {
            current_end_candidate
        };

        let b = current_start.format("%Y-%m-%d").to_string();
        let e = current_end.format("%Y-%m-%d").to_string();

        let resp = client.get_calendar(&b, &e, true).await?;

        for date_item in &resp.date_list {
            for cal_item in &date_item.calendar_item_list {
                if cal_item.status != "ORDER" {
                    continue;
                }
                let Some(corp) = &cal_item.corp_order_user else {
                    continue;
                };
                let Some(restaurants) = &corp.restaurant_item_list else {
                    continue;
                };

                let opening_name = cal_item
                    .opening_time
                    .as_ref()
                    .and_then(|o| o.name.as_deref())
                    .unwrap_or("");
                let meal_time = determine_meal_time(&cal_item.title, opening_name);

                for r_item in restaurants {
                    let r_name = r_item
                        .restaurant
                        .as_ref()
                        .and_then(|r| r.name.as_deref())
                        .unwrap_or("Unknown");

                    if let Some(dish_list) = &r_item.dish_item_list {
                        for d in dish_list {
                            all_orders.push(HistoricalOrder {
                                date: date_item.date.clone(),
                                meal_time: meal_time.clone(),
                                dish_name: d.dish.name.clone(),
                                restaurant_name: r_name.to_string(),
                                price_in_cent: d.dish.price_in_cent.unwrap_or(0),
                            });
                        }
                    }
                }
            }
        }

        current_start = current_start + chrono::Duration::days(chunk_days);
    }

    display::print_history(&all_orders, table);
    Ok(())
}
