use anyhow::{bail, Result};
use chrono::Local;

use crate::api::MeicanClient;
use crate::config;
use crate::display;
use crate::Meal;

fn build_client() -> Result<MeicanClient> {
    let session = config::load_session()?;
    Ok(MeicanClient::new(&session.cookies))
}

fn matches_meal(title: &str, meal: &Meal) -> bool {
    let t = title.to_lowercase();
    match meal {
        Meal::Breakfast => t.contains("早餐") || t.contains("breakfast"),
        Meal::Lunch => t.contains("午餐") || t.contains("lunch"),
        Meal::Dinner => t.contains("晚餐") || t.contains("dinner"),
    }
}

pub struct ResolvedTab {
    pub tab_unique_id: String,
    pub target_time: String,
}

/// Resolve tab by meal type from today's calendar.
/// If meal is None, picks the next AVAILABLE slot.
pub async fn resolve_tab_by_meal(
    client: &MeicanClient,
    meal: Option<&Meal>,
) -> Result<ResolvedTab> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let resp = client.get_calendar(&today, &today, false).await?;

    for date_item in &resp.date_list {
        for cal_item in &date_item.calendar_item_list {
            let Some(user_tab) = &cal_item.user_tab else {
                continue;
            };

            let should_match = match meal {
                Some(m) => matches_meal(&cal_item.title, m),
                None => cal_item.status == "AVAILABLE",
            };

            if should_match {
                let ts = cal_item
                    .target_time
                    .ok_or_else(|| anyhow::anyhow!("No target time for tab"))?;
                return Ok(ResolvedTab {
                    tab_unique_id: user_tab.unique_id.clone(),
                    target_time: ts.to_string(),
                });
            }
        }
    }

    match meal {
        Some(m) => bail!("No {:?} slot found in today's calendar", m),
        None => bail!("No available meal slot found in today's calendar"),
    }
}

/// Resolve tab: prefer --tab if given, otherwise resolve by meal.
async fn resolve_tab(
    client: &MeicanClient,
    meal: Option<&Meal>,
    tab: Option<&str>,
) -> Result<ResolvedTab> {
    if let Some(tab_id) = tab {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let resp = client.get_calendar(&today, &today, false).await?;

        for date_item in &resp.date_list {
            for cal_item in &date_item.calendar_item_list {
                let id = cal_item
                    .user_tab
                    .as_ref()
                    .map(|t| t.unique_id.as_str());
                if id == Some(tab_id) {
                    let ts = cal_item
                        .target_time
                        .ok_or_else(|| anyhow::anyhow!("No target time for tab"))?;
                    return Ok(ResolvedTab {
                        tab_unique_id: tab_id.to_string(),
                        target_time: ts.to_string(),
                    });
                }
            }
        }
        bail!("Tab ID '{}' not found in today's calendar", tab_id);
    }

    resolve_tab_by_meal(client, meal.or(None)).await
}

pub async fn today(table: bool) -> Result<()> {
    let client = build_client()?;
    let today = Local::now().format("%Y-%m-%d").to_string();
    let resp = client.get_calendar(&today, &today, true).await?;
    display::print_calendar(&resp, table);
    Ok(())
}

pub async fn calendar(begin_date: &str, end_date: &str, table: bool) -> Result<()> {
    let client = build_client()?;
    let resp = client.get_calendar(begin_date, end_date, true).await?;
    display::print_calendar(&resp, table);
    Ok(())
}

pub async fn dishes(meal: Option<Meal>, tab: Option<&str>, table: bool) -> Result<()> {
    let client = build_client()?;
    let resolved = resolve_tab(&client, meal.as_ref(), tab).await?;
    let resp = client
        .get_dishes(&resolved.tab_unique_id, &resolved.target_time)
        .await?;
    display::print_dishes(&resp, table);
    Ok(())
}

pub async fn restaurants(meal: Option<Meal>, tab: Option<&str>, table: bool) -> Result<()> {
    let client = build_client()?;
    let resolved = resolve_tab(&client, meal.as_ref(), tab).await?;
    let resp = client
        .get_restaurants(&resolved.tab_unique_id, &resolved.target_time)
        .await?;
    display::print_restaurants(&resp, table);
    Ok(())
}

pub async fn addresses(table: bool) -> Result<()> {
    let client = build_client()?;
    let resp = client.get_addresses("").await?;
    display::print_addresses(&resp, table);
    Ok(())
}
