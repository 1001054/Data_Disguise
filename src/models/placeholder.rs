use actix_web::web::Json;
use sqlx::FromRow;
use serde::{Deserialize, Serialize};
/// the location of the placeholder stored in database
#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct PlaceholderInfo {
    pub pred: Option<String>,
    pub table: Option<String>,
}
impl PlaceholderInfo {
    pub fn new(generate_placeholder: GeneratePlaceHolder, id: &str) -> PlaceholderInfo {
        PlaceholderInfo {
            pred: (generate_placeholder.primary_key_name.unwrap() + "=" + id).into(),
            table: generate_placeholder.table,
        }
    }
}
/// the input placeholder information from user or web
#[derive(Deserialize, Debug, Clone)]
pub struct GeneratePlaceHolder {
    pub table: Option<String>,
    pub primary_key_name: Option<String>,
    pub fields: Option<String>,
    pub field_values: Option<String>,
}
impl From<Json<GeneratePlaceHolder>> for GeneratePlaceHolder {
    fn from(placeholder_json: Json<GeneratePlaceHolder>) -> Self {
        GeneratePlaceHolder {
            table: placeholder_json.table.clone(),
            primary_key_name: placeholder_json.primary_key_name.clone(),
            fields: placeholder_json.fields.clone(),
            field_values: placeholder_json.field_values.clone(),
        }
    }
}