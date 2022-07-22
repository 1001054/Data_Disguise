use actix_web::web;
use actix_web::web::Json;
use serde::Deserialize;

/// which is the fundamental operation of the required disguise
/// three types are "removal", "modification", "decorrelation"
#[derive(Deserialize, Debug, Clone)]
pub struct Transformation {
    pub transform_type: Option<String>,
    pub table_name: Option<String>,
    pub predicate: Option<String>,
    pub foreign_key: Option<String>,
    pub changes: Option<String>
}
impl Transformation {
    /// get a new empty transformation
    pub fn new_empty() -> Self {
        Self {
            transform_type: None,
            table_name: None,
            predicate: None,
            foreign_key: None,
            changes: None
        }
    }
}

impl From<web::Json<Transformation>> for Transformation {
    fn from(json_transform: Json<Transformation>) -> Self {
        Transformation {
            transform_type: json_transform.transform_type.clone(),
            table_name: json_transform.table_name.clone(),
            predicate: json_transform.predicate.clone(),
            foreign_key: json_transform.foreign_key.clone(),
            changes: json_transform.changes.clone()
        }
    }
}