use actix_web::{HttpResponse, web};
use crate::dbaccess::target::generate_placeholder_db;
use crate::dbaccess::vault::generate_vault_db;
use crate::error::MyError;
use crate::models::placeholder::PlaceholderInfo;
use crate::models::vault::{GenerateVault, Vault};
use crate::state::AppState;

///
/// generate a new vault for a user
///
/// # Arguments
///
/// * `app_state`: the state of the server
/// * `generate_vault`: the data from user or web
///
/// returns: Result<HttpResponse<BoxBody>, MyError>
///
pub async fn generate_vault(
    app_state: web::Data<AppState>,
    generate_vault: web::Json<GenerateVault>
) -> Result<HttpResponse, MyError> {
    let vault_db = &app_state.vault_db;
    //generate the placeholder in the database
    let generate_placeholder = generate_vault.generate_placeholder.as_ref().unwrap().clone();
    //get the new placeholder's id and put it into vault
    let placeholder_id = generate_placeholder_db(&app_state.target_db, &generate_placeholder).await.unwrap();
    let vault = Vault {
        vault_id: generate_vault.vault_id.clone(),
        email: generate_vault.email.clone(),
        placeholder_info: Some(serde_json::to_string(&PlaceholderInfo {
            pred: Some(generate_placeholder.primary_key_name.unwrap().clone() + "=" + placeholder_id.as_str()),
            table: Some(generate_placeholder.table.unwrap().clone())
        }).unwrap())
    };
    //then generate the vault in the database
    generate_vault_db(vault_db, vault)
        .await
        .map(|msg| HttpResponse::Ok().json(msg) )
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::sync::Mutex;
    use actix_web::http::StatusCode;
    use actix_web::web;
    use sqlx::mysql::MySqlPoolOptions;
    use dotenv::dotenv;
    use crate::dbaccess::vault::generate_vault_db;
    use crate::handlers::vault::generate_vault;
    use crate::models::placeholder::{GeneratePlaceHolder, PlaceholderInfo};
    use crate::models::vault::{GenerateVault, Vault};
    use crate::state::AppState;

    #[ignore]
    #[actix_rt::test]
    async fn generate_vault_db_test() {
        //load the env variables
        dotenv().ok();
        //build the target database using the url in the .env
        //the target database is the database to disguise
        let target_database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set yet.");
        let vault_database_url = env::var("VAULT_DATABASE_URL").expect("VAULT_DATABASE_URL is not set yet.");
        let target_db = MySqlPoolOptions::new().connect(&target_database_url).await.unwrap();
        let vault_db = MySqlPoolOptions::new().connect(&vault_database_url).await.unwrap();
        //put the target database pool in the state
        let shared_data = web::Data::new(AppState {
            vault_db,
            target_db
        });
        let generate_placeholder = GeneratePlaceHolder {
            table: Some("contact_info".into()),
            primary_key_name: Some("contact_id".into()),
            fields: Some("name, email, disabled".into()),
            field_values: Some("'placeholder', '777', true".into())
        };
        let new_vault = GenerateVault {
            vault_id: Some("19".into()),
            email: Some("bea@mail.com".into()),
            generate_placeholder: Some(generate_placeholder)
        };
        let new_vault = web::Json(new_vault);
        let res = generate_vault(shared_data, new_vault).await;
        assert_eq!(res.is_ok(), true);
    }
}

