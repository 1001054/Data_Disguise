use actix_web::*;
use chrono::Local;
use sqlx::MySqlPool;
use crate::error::MyError;
use crate::models::requirement::*;
use crate::dbaccess::target::*;
use crate::dbaccess::vault::*;
use crate::models::placeholder::PlaceholderInfo;
use crate::state::AppState;


///
/// delete the user's information
/// but retaining the anonymized publications
///
/// # Arguments
///
/// * `app_state`: the state of the server
/// * `requirement`: the data from user or web
///
/// returns: Result<HttpResponse<BoxBody>, MyError>
///
pub async fn scrub_user(
    app_state: web::Data<AppState>,
    requirement: web::Json<Requirement>,
) -> Result<HttpResponse, MyError> {
    let target_pool = &app_state.target_db;
    let vault_pool = &app_state.vault_db;
    let disguise_name = requirement.disguise_name.as_ref().unwrap().to_lowercase();
    let vault_id = requirement.vault_id.as_ref().unwrap();
    let transformations = requirement.transformations.as_ref().unwrap();
    //check if the disguise name is right
    if disguise_name != String::from("userscrub") {
        return Err(MyError::InvalidInput("The disguise name is not correct.".into()));
    }
    //check if the vault exists in database for this user
    let vault = get_vault_by_id_db(vault_pool, vault_id).await?;
    //get the placeholder info from the vault
    let placeholder_info: PlaceholderInfo = serde_json::from_str(vault.placeholder_info.unwrap().as_str()).unwrap();
    let placeholder_pred = placeholder_info.pred.as_ref().unwrap().as_str();

    //get the original state of the target
    let original = get_targets_db(target_pool, &transformations).await?;

    //execute the transformations to the target
    //and get all the changes of the transformations
    let all_changes = execute_transformations_db(placeholder_pred, target_pool, transformations).await?;

    //upload this disguise into the vault
    upload_disguise_db(vault_pool, &requirement, original, all_changes).await?;

    Ok(HttpResponse::Ok().json("The policy has been applied.".to_string()))
}

///
/// anonymize the contributions of the user
///
/// # Arguments
///
/// * `app_state`: the state of the server
/// * `requirement`: the data from user or web
///
/// returns: Result<HttpResponse<BoxBody>, MyError>
///
pub async fn anonymize(
    app_state: web::Data<AppState>,
    requirement: web::Json<Requirement>
) -> Result<HttpResponse, MyError> {
    let target_pool = &app_state.target_db;
    let vault_pool = &app_state.vault_db;
    let disguise_name = requirement.disguise_name.as_ref().unwrap().to_lowercase();
    let vault_id = requirement.vault_id.as_ref().unwrap();
    let transformations = requirement.transformations.as_ref().unwrap();
    //check if the disguise name is right
    if disguise_name != String::from("anonymize") {
        return Err(MyError::InvalidInput("The disguise name is not correct.".into()));
    }
    //check if the vault exists in database for this user
    let vault = get_vault_by_id_db(vault_pool, vault_id).await?;
    //get the placeholder info from the vault
    let placeholder_info: PlaceholderInfo = serde_json::from_str(vault.placeholder_info.unwrap().as_str()).unwrap();
    let placeholder_pred = placeholder_info.pred.as_ref().unwrap().as_str();

    //get the original state of the target
    let original = get_targets_db(target_pool, &transformations).await?;

    //execute the transformations to the target
    //and get all the changes of the transformations
    let all_changes = execute_transformations_db(placeholder_pred, target_pool, transformations).await?;

    //upload this disguise into the vault
    upload_disguise_db(vault_pool, &requirement, original, all_changes).await?;

    Ok(HttpResponse::Ok().json("The policy has been applied.".to_string()))
}

///
/// delete the old users and their publications
/// into the vault
///
/// # Arguments
///
/// * `app_state`: the state of the server
/// * `requirement`: the data from the user or the web
///
/// returns: Result<HttpResponse<BoxBody>, MyError>
///
pub async fn expiration(
    app_state: web::Data<AppState>,
    requirement: web::Json<Requirement>
) -> Result<HttpResponse, MyError> {
    let target_pool = &app_state.target_db;
    let vault_pool = &app_state.vault_db;
    let disguise_name = requirement.disguise_name.as_ref().unwrap().to_lowercase();
    let vault_id = requirement.vault_id.as_ref().unwrap();
    let delete_age = requirement.delete_age.unwrap();
    let transformations = requirement.transformations.as_ref().unwrap();
    //check if the disguise name is right
    if disguise_name != String::from("expiration") {
        return Err(MyError::InvalidInput("The disguise name is not correct.".into()));
    }
    //check if the vault exists in database for this user
    let vault = get_vault_by_id_db(vault_pool, vault_id).await?;
    //get the placeholder info from the vault
    let placeholder_info: PlaceholderInfo = serde_json::from_str(vault.placeholder_info.unwrap().as_str()).unwrap();
    let placeholder_pred = placeholder_info.pred.as_ref().unwrap().as_str();
    //transfer the transformations
    let transformations = transfer_transformations(target_pool, transformations, delete_age).await?;

    //get the original state of the target
    let original = get_targets_db(target_pool, &transformations).await?;

    //execute the transformations to the target
    //and get all the changes of the transformations
    let all_changes = execute_transformations_db(placeholder_pred, target_pool, &transformations).await?;

    //upload this disguise into the vault
    upload_disguise_db(vault_pool, &requirement, original, all_changes).await?;

    Ok(HttpResponse::Ok().json("The policy has been applied.".to_string()))
}

///
/// clear the old disguise data in all the vaults
/// they will be deleted forever
///
/// # Arguments
///
/// * `app_state`: the state of the server
/// * `requirement`: the data from user or web
///
/// returns: Result<HttpResponse<BoxBody>, MyError>
///
pub async fn clear_vault(
    app_state: web::Data<AppState>,
    requirement: web::Json<Requirement>,
) -> Result<HttpResponse, MyError> {
    let target_pool = &app_state.target_db;
    let vault_pool = &app_state.vault_db;
    let age = requirement.delete_age;
    let disguise_name = requirement.disguise_name.as_ref().unwrap().to_lowercase();
    //check if the disguise name is right
    if disguise_name != String::from("clearvault") {
        return Err(MyError::InvalidInput("The disguise name is not correct.".into()));
    }
    //check which type the developer want to clear
    match age {
        //by name
        None => {
            let name = requirement.delete_name.as_ref().unwrap();
            let vault_id = requirement.vault_id.as_ref().unwrap();
            delete_disguise_by_vault_id_and_name_db(target_pool, vault_pool, vault_id, name).await?;
        }
        //by age
        Some(age) => {
            delete_disguise_by_age_db(target_pool, vault_pool, age).await?;
        }
    }
    Ok(HttpResponse::Ok().json("The policy has been applied.".to_string()))
}

///
/// recover the applied disguise from the vault
/// and this disguise history will be deleted as well
///
/// # Arguments
///
/// * `app_state`: the state of the server
/// * `requirement`: the data from user or the web
///
/// returns: Result<HttpResponse<BoxBody>, MyError>
///
pub async fn recover_disguise(
    app_state: web::Data<AppState>,
    requirement: web::Json<Requirement>,
) -> Result<HttpResponse, MyError> {
    let target_db = &app_state.target_db;
    let vault_db = &app_state.vault_db;
    let disguise_type = requirement.disguise_name.as_ref().unwrap();
    let vault_id = requirement.vault_id.as_ref().unwrap();

    //download disguise from vault
    let disguise = download_disguise_db(vault_db, disguise_type, vault_id).await?;

    //recover the target
    recover_db(target_db, &disguise).await?;

    //delete the disguise in the vault
    delete_disguise_db(vault_db, disguise_type, vault_id).await?;

    Ok(HttpResponse::Ok().json("The disguise has been recovered."))
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::future::Future;
    use std::sync::Mutex;
    use std::time::Duration;
    use actix_web::http::StatusCode;
    use actix_web::web;
    use actix_web::web::Json;
    use chrono::Local;
    use sqlx::mysql::MySqlPoolOptions;
    use dotenv::dotenv;
    use crate::error::MyError;
    use crate::handlers::disguise::*;
    use crate::models::requirement::Requirement;
    use crate::models::transformation::Transformation;
    use crate::state::AppState;

    #[ignore]
    #[actix_rt::test]
    async fn scrub_user_test() {

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
            target_db,
        });
        //create the transformations and requirement
        let decorrelate = Transformation {
            transform_type: Some("decorrelation".into()),
            table_name: Some("review".into()),
            predicate: Some("contact_id=19".into()),
            foreign_key: Some("contact_id".into()),
            changes: None,
        };
        let removal = Transformation {
            transform_type: Some("removal".into()),
            table_name: Some("contact_info".into()),
            predicate: Some("contact_id=19".into()),
            foreign_key: None,
            changes: None,
        };
        let requirement = Requirement {
            disguise_name: Some("userscrub".into()),
            vault_id: Some("19".into()),
            delete_age: None,
            delete_name: None,
            transformations: Some(vec![decorrelate, removal]),
        };
        //covert the requirement into json
        // let requirement = web::Json(&requirement);

        
        //the requirement of recover
        let requirement_recover = Requirement {
            disguise_name: Some("userscrub".to_string()),
            vault_id: Some("19".to_string()),
            delete_age: None,
            delete_name: None,
            transformations: None
        };
        // let requirement_recover = web::Json(&requirement_recover);

        let mut i = 0;
        // test this policy repetitively
        while i < 4 {
            //sleep 5s
            std::thread::sleep(Duration::from_secs(1));

            //start the time
            let start = Local::now();


            let res = scrub_user(shared_data.clone(), Json(requirement.clone())).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);

            //to test the time
            let time = Local::now() - start;
            println!("{:?}", time.num_milliseconds());
            
            recover_disguise(shared_data.clone(), Json(requirement_recover.clone())).await.unwrap();
            
            i += 1;
        }

    }
}