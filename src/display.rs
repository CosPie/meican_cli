use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};

use crate::models::*;

fn status_color(status: &str) -> Color {
    match status {
        "AVAILABLE" => Color::Green,
        "ORDER" => Color::Cyan,
        "CLOSED" => Color::DarkGrey,
        _ => Color::White,
    }
}

fn format_price(price_in_cent: i64) -> String {
    format!("¥{:.2}", price_in_cent as f64 / 100.0)
}

pub fn print_calendar(resp: &CalendarResponse, table: bool) {
    if !table {
        println!("{}", serde_json::to_string_pretty(resp).unwrap_or_default());
        return;
    }

    if resp.date_list.is_empty() {
        println!("{}", "No calendar data found.".yellow());
        return;
    }

    let mut t = Table::new();
    t.load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Date"),
            Cell::new("Meal"),
            Cell::new("Status"),
            Cell::new("Tab ID"),
            Cell::new("Ordered"),
        ]);

    for date_item in &resp.date_list {
        for cal_item in &date_item.calendar_item_list {
            let tab_id = cal_item
                .user_tab
                .as_ref()
                .map(|t| t.unique_id.as_str())
                .unwrap_or("-");

            let ordered = extract_ordered_dishes(cal_item);
            let color = status_color(&cal_item.status);

            t.add_row(vec![
                Cell::new(&date_item.date),
                Cell::new(&cal_item.title),
                Cell::new(&cal_item.status).fg(color),
                Cell::new(tab_id),
                Cell::new(&ordered),
            ]);
        }
    }

    println!("{t}");
}

fn extract_ordered_dishes(item: &CalendarItem) -> String {
    let Some(corp) = &item.corp_order_user else {
        return "-".to_string();
    };
    let Some(restaurants) = &corp.restaurant_item_list else {
        return "-".to_string();
    };

    let mut dishes = Vec::new();
    for r in restaurants {
        let r_name = r
            .restaurant
            .as_ref()
            .and_then(|r| r.name.as_deref())
            .unwrap_or("?");
        if let Some(dish_list) = &r.dish_item_list {
            for d in dish_list {
                let price = d
                    .dish
                    .price_in_cent
                    .map(|p| format_price(p))
                    .unwrap_or_default();
                dishes.push(format!("{} ({}) {}", d.dish.name, r_name, price));
            }
        }
    }

    if dishes.is_empty() {
        "-".to_string()
    } else {
        dishes.join(", ")
    }
}

pub fn print_dishes(resp: &DishesResponse, table: bool) {
    if !table {
        println!("{}", serde_json::to_string_pretty(resp).unwrap_or_default());
        return;
    }

    let dishes = resp.others_regular_dish_list.as_deref().unwrap_or(&[]);
    if dishes.is_empty() {
        println!("{}", "No dishes available.".yellow());
        return;
    }

    let mut t = Table::new();
    t.load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("#"),
            Cell::new("Dish ID"),
            Cell::new("Name"),
            Cell::new("Price"),
            Cell::new("Restaurant"),
        ]);

    for (i, dish) in dishes.iter().enumerate() {
        let price = dish
            .price_in_cent
            .map(|p| format_price(p))
            .unwrap_or_else(|| "-".into());
        let restaurant = dish
            .restaurant
            .as_ref()
            .and_then(|r| r.name.as_deref())
            .unwrap_or("-");
        let dish_id = match &dish.id {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };

        t.add_row(vec![
            Cell::new(i + 1),
            Cell::new(&dish_id),
            Cell::new(&dish.name),
            Cell::new(&price),
            Cell::new(restaurant),
        ]);
    }

    println!("{}", format!("Found {} dishes:", dishes.len()).green());
    println!("{t}");
}

pub fn print_restaurants(resp: &RestaurantsResponse, table: bool) {
    if !table {
        println!("{}", serde_json::to_string_pretty(resp).unwrap_or_default());
        return;
    }

    let restaurants = resp.restaurant_list.as_deref().unwrap_or(&[]);
    if restaurants.is_empty() {
        println!("{}", "No restaurants available.".yellow());
        return;
    }

    let mut t = Table::new();
    t.load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("#"),
            Cell::new("ID"),
            Cell::new("Name"),
            Cell::new("Rating"),
        ]);

    for (i, r) in restaurants.iter().enumerate() {
        let rating = r
            .rating
            .map(|v| format!("{:.1}", v))
            .unwrap_or_else(|| "-".into());

        t.add_row(vec![
            Cell::new(i + 1),
            Cell::new(&r.unique_id),
            Cell::new(&r.name),
            Cell::new(&rating),
        ]);
    }

    println!(
        "{}",
        format!("Found {} restaurants:", restaurants.len()).green()
    );
    println!("{t}");
}

pub fn print_addresses(resp: &AddressResponse, table: bool) {
    if !table {
        println!("{}", serde_json::to_string_pretty(resp).unwrap_or_default());
        return;
    }

    let data = match &resp.data {
        Some(d) => d,
        None => {
            println!("{}", "No address data.".yellow());
            return;
        }
    };

    let mut t = Table::new();
    t.load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("#"),
            Cell::new("ID"),
            Cell::new("Name"),
            Cell::new("Address"),
        ]);

    let mut idx = 1;

    if let Some(addr_list) = &data.address_list {
        for wrapper in addr_list {
            if let Some(addr) = &wrapper.final_value {
                let id = addr.unique_id.as_deref().unwrap_or("-");
                let name = addr.name.as_deref().unwrap_or("-");
                let address = addr.address.as_deref().unwrap_or("-");
                t.add_row(vec![
                    Cell::new(idx),
                    Cell::new(id),
                    Cell::new(name),
                    Cell::new(address),
                ]);
                idx += 1;
            }
        }
    }

    if let Some(recent) = &data.recent_list {
        if !recent.is_empty() {
            for addr in recent {
                let id = addr.unique_id.as_deref().unwrap_or("-");
                let name = addr.name.as_deref().unwrap_or("-");
                let address = addr.address.as_deref().unwrap_or("-");
                t.add_row(vec![
                    Cell::new(idx),
                    Cell::new(id),
                    Cell::new(name),
                    Cell::new(address),
                ]);
                idx += 1;
            }
        }
    }

    if idx == 1 {
        println!("{}", "No addresses found.".yellow());
    } else {
        println!("{}", format!("Found {} addresses:", idx - 1).green());
        println!("{t}");
    }
}

pub fn print_history(orders: &[HistoricalOrder], table: bool) {
    if !table {
        println!("{}", serde_json::to_string_pretty(orders).unwrap_or_default());
        return;
    }

    if orders.is_empty() {
        println!("{}", "No order history found.".yellow());
        return;
    }

    let mut t = Table::new();
    t.load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Date"),
            Cell::new("Meal"),
            Cell::new("Dish"),
            Cell::new("Restaurant"),
            Cell::new("Price"),
        ]);

    for order in orders {
        let meal_color = match order.meal_time.as_str() {
            "BREAKFAST" => Color::Yellow,
            "LUNCH" => Color::Green,
            "DINNER" => Color::Magenta,
            _ => Color::White,
        };

        t.add_row(vec![
            Cell::new(&order.date),
            Cell::new(&order.meal_time).fg(meal_color),
            Cell::new(&order.dish_name),
            Cell::new(&order.restaurant_name),
            Cell::new(format_price(order.price_in_cent)),
        ]);
    }

    println!(
        "{}",
        format!("Order history ({} items):", orders.len()).green()
    );
    println!("{t}");
}

pub fn print_order_result(resp: &OrderResponse, table: bool) {
    if !table {
        println!("{}", serde_json::to_string_pretty(resp).unwrap_or_default());
        return;
    }

    match resp.status.as_deref() {
        Some("SUCCESSFUL") => {
            println!("{}", "Order placed successfully!".green().bold());
            if let Some(order) = &resp.order {
                if let Some(id) = &order.unique_id {
                    println!("  Order ID: {}", id.cyan());
                }
            }
        }
        _ => {
            println!("{}", "Order failed!".red().bold());
            if let Some(msg) = &resp.message {
                println!("  Reason: {}", msg);
            }
        }
    }
}
