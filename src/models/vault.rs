use actix_web::web::Json;
use sqlx::FromRow;
use serde::Deserialize;
use crate::models::placeholder::GeneratePlaceHolder;

/// which from the web app or the user input
#[derive(Deserialize, Debug, Clone)]
pub struct GenerateVault {
    pub vault_id: Option<String>,
    pub email: Option<String>,
    pub generate_placeholder: Option<GeneratePlaceHolder>
}
impl From<Json<GenerateVault>> for GenerateVault {
    fn from(vault_json: Json<GenerateVault>) -> Self {
        GenerateVault {
            vault_id: vault_json.vault_id.clone(),
            email: vault_json.email.clone(),
            generate_placeholder: vault_json.generate_placeholder.clone()
        }
    }
}
/// which from or to the database
#[derive(Debug, Clone, FromRow)]
pub struct Vault {
    pub vault_id: Option<String>,
    pub email: Option<String>,
    pub placeholder_info: Option<String>
}
/// which from or to the database
/// stored in the vaults
#[derive(Debug, Clone, FromRow)]
pub struct DisguiseFromDB {
    pub disguise_id: Option<i32>,
    pub time: Option<String>,
    pub vault_id: Option<String>,
    pub disguise_type: Option<String>
}
/// applied disguise stored in database
#[derive(Debug, Clone)]
pub struct Disguise {
    pub disguise_id: Option<i32>,
    pub time: Option<String>,
    pub vault_id: Option<String>,
    pub disguise_type: Option<String>,
    pub functions: Option<Vec<Function>>
}
impl From<DisguiseFromDB> for Disguise {
    fn from(disguise_from_db: DisguiseFromDB) -> Self {
        Disguise {
            disguise_id: disguise_from_db.disguise_id,
            time: disguise_from_db.time,
            vault_id: disguise_from_db.vault_id,
            disguise_type: disguise_from_db.disguise_type,
            functions: None
        }
    }
}
/// which from or to the database
/// make up the disguise
#[derive(Debug, Clone, FromRow)]
pub struct Function {
    pub disguise_id: Option<i32>,
    pub function_type: Option<String>,
    pub table_name: Option<String>,
    pub predicate: Option<String>,
    pub original: Option<String>,
    pub updated: Option<String>
}