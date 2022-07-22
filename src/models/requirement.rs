use actix_web::web;
use actix_web::web::Json;
use serde::Deserialize;
use crate::models::transformation::Transformation;

/// which comes from the web app or the users
/// to describe the disguise
#[derive(Deserialize, Debug, Clone)]
pub struct Requirement {
    pub disguise_name: Option<String>,
    pub vault_id: Option<String>,
    pub delete_age: Option<i64>,
    pub delete_name: Option<String>,
    pub transformations: Option<Vec<Transformation>>,
}

impl From<web::Json<Requirement>> for Requirement {
    fn from(json_requirement: Json<Requirement>) -> Self {
        Requirement {
            disguise_name: json_requirement.disguise_name.clone(),
            vault_id: json_requirement.vault_id.clone(),
            delete_age: json_requirement.delete_age.clone(),
            delete_name: json_requirement.delete_name.clone(),
            transformations: json_requirement.transformations.clone()
        }
    }
}