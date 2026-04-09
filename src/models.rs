use serde::{Deserialize, Serialize};

// ============================================================================
// Calendar
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarResponse {
    pub date_list: Vec<CalendarDate>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDate {
    pub date: String,
    pub calendar_item_list: Vec<CalendarItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CalendarItem {
    pub title: String,
    pub status: String,
    pub target_time: Option<i64>,
    pub user_tab: Option<UserTab>,
    pub opening_time: Option<OpeningTime>,
    pub corp_order_user: Option<CorpOrderUser>,
    pub corp: Option<Corp>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct UserTab {
    pub unique_id: String,
    pub name: Option<String>,
    pub corp: Option<Corp>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningTime {
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CorpOrderUser {
    pub unique_id: String,
    pub user_address_unique_id: Option<String>,
    pub corp_address: Option<CorpAddress>,
    pub corp: Option<Corp>,
    pub restaurant_item_list: Option<Vec<RestaurantOrderItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Corp {
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CorpAddress {
    pub unique_id: Option<String>,
    pub name: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct RestaurantOrderItem {
    pub unique_id: Option<String>,
    pub restaurant: Option<RestaurantRef>,
    pub dish_item_list: Option<Vec<DishOrderItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestaurantRef {
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DishOrderItem {
    pub dish: OrderedDish,
    pub count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct OrderedDish {
    pub id: serde_json::Value,
    pub name: String,
    pub price_in_cent: Option<i64>,
}

// ============================================================================
// Address
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressResponse {
    pub data: Option<AddressData>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressData {
    pub address_list: Option<Vec<AddressWrapper>>,
    pub recent_list: Option<Vec<Address>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressWrapper {
    pub final_value: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Address {
    pub unique_id: Option<String>,
    pub name: Option<String>,
    pub address: Option<String>,
}

// ============================================================================
// Restaurant
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestaurantsResponse {
    pub restaurant_list: Option<Vec<Restaurant>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Restaurant {
    pub unique_id: String,
    pub name: String,
    pub rating: Option<f64>,
}

// ============================================================================
// Dish
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DishesResponse {
    pub others_regular_dish_list: Option<Vec<Dish>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dish {
    pub id: serde_json::Value,
    pub name: String,
    pub price_in_cent: Option<i64>,
    pub restaurant: Option<DishRestaurant>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DishRestaurant {
    pub unique_id: Option<String>,
    pub name: Option<String>,
}

// ============================================================================
// Order
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub status: Option<String>,
    pub order: Option<OrderRef>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRef {
    pub unique_id: Option<String>,
}

// ============================================================================
// History (client-side aggregated)
// ============================================================================

#[derive(Debug, Serialize)]
pub struct HistoricalOrder {
    pub date: String,
    pub meal_time: String,
    pub dish_name: String,
    pub restaurant_name: String,
    pub price_in_cent: i64,
}

// ============================================================================
// Session (local persistence)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub cookies: String,
    pub created_at: String,
    pub username: String,
}
