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
    pub namespace: String,
}

/// Resolve tab by meal type from a given date's calendar.
/// If meal is None, picks the next AVAILABLE slot.
/// If date is None, defaults to today.
pub async fn resolve_tab_by_meal(
    client: &MeicanClient,
    meal: Option<&Meal>,
    date: Option<&str>,
) -> Result<ResolvedTab> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let target_date = date.unwrap_or(&today);
    let resp = client.get_calendar(target_date, target_date, false).await?;

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
                let target_time = match meal {
                    Some(Meal::Breakfast) => format!("{} 07:00", target_date),
                    Some(Meal::Lunch) => format!("{} 09:00", target_date),
                    Some(Meal::Dinner) => format!("{} 15:00", target_date),
                    None => {
                        // No meal specified — derive from title
                        if cal_item.title.contains("早餐") || cal_item.title.to_lowercase().contains("breakfast") {
                            format!("{} 07:00", target_date)
                        } else if cal_item.title.contains("午餐") || cal_item.title.to_lowercase().contains("lunch") {
                            format!("{} 09:00", target_date)
                        } else {
                            format!("{} 15:00", target_date)
                        }
                    }
                };
                let namespace = user_tab.corp.as_ref()
                    .and_then(|c| c.namespace.clone())
                    .unwrap_or_default();
                return Ok(ResolvedTab {
                    tab_unique_id: user_tab.unique_id.clone(),
                    target_time,
                    namespace,
                });
            }
        }
    }

    match meal {
        Some(m) => bail!("No {:?} slot found in calendar for {}", m, date.unwrap_or(&today)),
        None => bail!("No available meal slot found in calendar for {}", date.unwrap_or(&today)),
    }
}

/// Resolve tab: prefer --tab if given, otherwise resolve by meal.
pub async fn resolve_tab(
    client: &MeicanClient,
    meal: Option<&Meal>,
    tab: Option<&str>,
    date: Option<&str>,
) -> Result<ResolvedTab> {
    if let Some(tab_id) = tab {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let target_date = date.unwrap_or(&today);
        let resp = client.get_calendar(target_date, target_date, false).await?;

        for date_item in &resp.date_list {
            for cal_item in &date_item.calendar_item_list {
                let id = cal_item
                    .user_tab
                    .as_ref()
                    .map(|t| t.unique_id.as_str());
                if id == Some(tab_id) {
                    let target_time = if cal_item.title.contains("早餐") || cal_item.title.to_lowercase().contains("breakfast") {
                        format!("{} 07:00", target_date)
                    } else if cal_item.title.contains("午餐") || cal_item.title.to_lowercase().contains("lunch") {
                        format!("{} 09:00", target_date)
                    } else {
                        format!("{} 15:00", target_date)
                    };
                    let namespace = cal_item.user_tab.as_ref()
                        .and_then(|t| t.corp.as_ref())
                        .and_then(|c| c.namespace.clone())
                        .unwrap_or_default();
                    return Ok(ResolvedTab {
                        tab_unique_id: tab_id.to_string(),
                        target_time,
                        namespace,
                    });
                }
            }
        }
        bail!("Tab ID '{}' not found in calendar for {}", tab_id, target_date);
    }

    resolve_tab_by_meal(client, meal.or(None), date).await
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

pub async fn dishes(meal: Option<Meal>, tab: Option<&str>, date: Option<&str>, table: bool) -> Result<()> {
    let client = build_client()?;
    let resolved = resolve_tab(&client, meal.as_ref(), tab, date).await?;
    let resp = client
        .get_dishes(&resolved.tab_unique_id, &resolved.target_time)
        .await?;
    display::print_dishes(&resp, table);
    Ok(())
}

pub async fn restaurants(meal: Option<Meal>, tab: Option<&str>, date: Option<&str>, table: bool) -> Result<()> {
    let client = build_client()?;
    let resolved = resolve_tab(&client, meal.as_ref(), tab, date).await?;
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
