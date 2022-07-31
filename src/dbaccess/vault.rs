use chrono::{Duration, Local};
use sqlx::mysql::MySqlRow;
use sqlx::MySqlPool;
use crate::dbaccess::target::delete_decorrelated_targets_db;
use crate::error::MyError;
use crate::models::requirement::Requirement;
use crate::models::target::Target;
use crate::models::vault::{Disguise, DisguiseFromDB, Function, Vault};


///
/// generate a new vault for the user into the "vault" table
/// of the server's database
///
/// # Arguments
///
/// * `vault_db`: the server's database
/// * `vault`: the vault to generate
///
/// returns: Result<String, MyError>
///
///
pub async fn generate_vault_db(vault_db: &MySqlPool, vault: Vault) -> Result<String, MyError> {
    sqlx::query("INSERT INTO vault (vault_id, email, placeholder_info) values (?, ? , ?)")
        .bind(vault.vault_id.clone())
        .bind(vault.email.clone())
        .bind(vault.placeholder_info.clone())
        .execute(vault_db)
        .await?;

    Ok("The vault has been generated.".into())
}

///
/// get the vault by id
///
/// # Arguments
///
/// * `vault_db`: the server's database
/// * `vault_id`: the id of vault to generate
///
/// returns: Result<Vault, MyError>
///
pub async fn get_vault_by_id_db(vault_db: &MySqlPool, vault_id: &str) -> Result<Vault, MyError> {
    let res: Result<Vault, MyError> = sqlx::query_as("SELECT * FROM vault WHERE vault_id = ?")
        .bind(vault_id)
        .fetch_one(vault_db)
        .await
        .map_err(|_err| MyError::NotFound("Vault id is not found.".into()));
    res
}

///
/// get the vault by email
///
/// # Arguments
///
/// * `vault_db`: the server's database
/// * `email`: the email of vault's owner
///
/// returns: Result<Vault, MyError>
///
pub async fn get_vault_by_email_db(vault_db: &MySqlPool, email: &str) -> Result<Vault, MyError> {
    let res: Result<Vault, MyError> = sqlx::query_as("SELECT * FROM vault WHERE email = ?")
        .bind(email)
        .fetch_one(vault_db)
        .await
        .map_err(|_err| MyError::NotFound("Vault email is not found.".into()));
    res
}

///
/// upload the applied disguise into vault
///
/// # Arguments
///
/// * `vault_pool`: the server's database
/// * `requirement`: the disguise's requirement
/// * `originals`: the original state of the targets of transformations
/// * `updated`: the changes of the transformations
///
/// returns: Result<String, MyError>
///
pub async fn upload_disguise_db(
    vault_pool: &MySqlPool,
    requirement: &Requirement,
    originals: Vec<Vec<Target>>,
    updated: Vec<String>,
) -> Result<String, MyError> {
    let time = Local::now().to_string();
    let disguise_type = requirement.disguise_name.as_ref().unwrap().clone();
    let vault_id = requirement.vault_id.as_ref().unwrap().clone();
    let transformations = requirement.transformations.as_ref().unwrap();

    //insert the disguise into database
    let disguise_id = sqlx::query("INSERT INTO disguise (time, vault_id, disguise_type) values (?, ?, ?)")
        .bind(time)
        .bind(vault_id)
        .bind(disguise_type)
        .execute(vault_pool)
        .await?
        .last_insert_id();

    //insert the functions into database
    for (i, transformation) in transformations.iter().enumerate() {
        let transform_type = transformation.transform_type.as_ref().unwrap();
        let table = transformation.table_name.as_ref().unwrap();
        let original_vec = &originals[i];
        //iterate all the rows affected by the same transformation
        for original in original_vec {
            let key = original.primary_key().unwrap();
            //insert the functions into the database
            let sql = "INSERT INTO function (disguise_id, function_type, table_name, predicate, original, updated) values (?, ?, ?, ?, ?, ?)";
            let predicate = String::from(key.field_name.unwrap()) + "=" + key.field_value.unwrap().as_str();
            let original_values = original.field_values().unwrap();
            sqlx::query(sql)
                .bind(disguise_id)
                .bind(transform_type)
                .bind(table)
                .bind(predicate)
                .bind(original_values)
                .bind(updated.get(i))
                .execute(vault_pool)
                .await?;
        }
    }

    Ok("".to_string())
}

///
/// download the disguise in vault for recovering
///
/// # Arguments
///
/// * `vault_pool`: the server's database
/// * `disguise_name`: the name of the disguise
/// * `vault_id`: the id of the vault
///
/// returns: Result<Disguise, MyError>
///
pub async fn download_disguise_db(
    vault_pool: &MySqlPool,
    disguise_name: &str,
    vault_id: &str,
) -> Result<Disguise, MyError> {
    //get disguise by id and type
    let sql = "SELECT * FROM disguise WHERE disguise_type=? and vault_id=?";
    let disguise: DisguiseFromDB = sqlx::query_as(sql)
        .bind(disguise_name)
        .bind(vault_id)
        .fetch_one(vault_pool)
        .await?;
    //get functions by disguise id
    let sql = "SELECT * FROM function WHERE disguise_id=?";
    let functions: Vec<Function> = sqlx::query_as(sql)
        .bind(disguise.disguise_id)
        .fetch_all(vault_pool)
        .await?;
    let mut disguise: Disguise = disguise.into();
    disguise.functions = Some(functions);
    Ok(disguise)
}

pub async fn upload_disguise_object_db(
    vault_pool: &MySqlPool,
    disguise: &Disguise
) -> Result<String, MyError> {
    let time = disguise.time.as_ref().unwrap().as_str();
    let vault_id = disguise.vault_id.as_ref().unwrap().as_str();
    let disguise_type = disguise.disguise_type.as_ref().unwrap().as_str();
    let functions = disguise.functions.as_ref().unwrap().clone();

    let sql = "INSERT INTO disguise (time, vault_id, disguise_type) VALUES (?, ?, ?)";
    //insert the disguise into database
    let disguise_id = sqlx::query(sql)
        .bind(time)
        .bind(vault_id)
        .bind(disguise_type)
        .execute(vault_pool)
        .await?
        .last_insert_id();

    for function in functions {
        let transform_type = function.function_type.as_ref().unwrap().as_str();
        let table = function.table_name.as_ref().unwrap().as_str();
        let predicate = function.predicate.as_ref().unwrap().as_str();
        let original_values = function.original.as_ref().unwrap().as_str();
        let updated = function.updated.as_ref().unwrap().as_str();

        let sql = "INSERT INTO function (disguise_id, function_type, table_name, predicate, original, updated) VALUES(?, ?, ?, ?, ?, ?)";
        sqlx::query(sql)
            .bind(disguise_id)
            .bind(transform_type)
            .bind(table)
            .bind(predicate)
            .bind(original_values)
            .bind(updated)
            .execute(vault_pool)
            .await?;
    }

    Ok("The disguise object has been uploaded.".to_string())
}

///
/// delete the disguise in the vault after recover
///
/// # Arguments
///
/// * `vault_pool`: the server's database
/// * `disguise_name`: the name of the disguise
/// * `vault_id`: the id of the vault
///
/// returns: Result<String, MyError>
///
pub async fn delete_disguise_db(
    vault_pool: &MySqlPool,
    disguise_name: &str,
    vault_id: &str,
) -> Result<String, MyError> {
    //get disguise by id and type
    let sql = "SELECT * FROM disguise WHERE disguise_type=? and vault_id=?";
    let disguise: DisguiseFromDB = sqlx::query_as(sql)
        .bind(disguise_name)
        .bind(vault_id)
        .fetch_one(vault_pool)
        .await?;
    //delete functions by disguise id
    let sql = "DELETE FROM function WHERE disguise_id=?";
    sqlx::query(sql)
        .bind(disguise.disguise_id)
        .execute(vault_pool)
        .await?;
    //delete the disguise
    let sql = "DELETE FROM disguise WHERE disguise_id=?";
    sqlx::query(sql)
        .bind(disguise.disguise_id)
        .execute(vault_pool)
        .await?;
    Ok("The disguise in the vault has been deleted.".to_string())
}

///
/// delete the old disguises in the vault
/// by the number of the years between right now and the applied time
/// And the decorrelated publications in application's database will be deleted
///
/// # Arguments
///
/// * `vault_pool`: the server's database
/// * `age`: the number of the years between right now and the applied time
///
/// returns: Result<String, MyError>
///
pub async fn delete_disguise_by_age_db(
    target_pool: &MySqlPool,
    vault_pool: &MySqlPool,
    age: i64,
) -> Result<String, MyError> {
    let delete_age = Local::now() - Duration::days(365 * age);
    //get disguise by id and type
    let sql = "SELECT * FROM disguise WHERE time<?";
    let disguises: Vec<DisguiseFromDB> = sqlx::query_as(sql)
        .bind(delete_age)
        .fetch_all(vault_pool)
        .await?;
    //delete functions by disguise id
    for disguise in disguises {
        //get the decorrelated publications' predicate
        let sql = "SELECT * FROM function WHERE disguise_id=? and function_type=?";
        let functions: Vec<Function> = sqlx::query_as(sql)
            .bind(disguise.disguise_id)
            .bind("decorrelation")
            .fetch_all(vault_pool)
            .await?;
        //destroy them
        for function in functions {
            delete_decorrelated_targets_db(
                target_pool,
                function.table_name.unwrap().as_str(),
                function.predicate.unwrap().as_str()
            ).await?;
        }
        //then delete the functions in vault
        let sql = "DELETE FROM function WHERE disguise_id=?";
        sqlx::query(sql)
            .bind(disguise.disguise_id)
            .execute(vault_pool)
            .await?;
    }
    //delete the disguise
    let sql = "DELETE FROM disguise WHERE time<?";
    sqlx::query(sql)
        .bind(delete_age)
        .execute(vault_pool)
        .await?;
    Ok("The old data in vault has been deleted.".to_string())
}

///
/// delete the old disguises in the vault by name
/// And the decorrelated publications in application's database will be deleted
///
/// # Arguments
///
/// * `vault_pool`: the server's database
/// * `name`: the applied disguise name
///
/// returns: Result<String, MyError>
///
pub async fn delete_disguise_by_vault_id_and_name_db(
    target_pool: &MySqlPool,
    vault_pool: &MySqlPool,
    vault_id: &str,
    name: &str,
) -> Result<String, MyError> {
    //get disguise by id and type
    let sql = "SELECT * FROM disguise WHERE vault_id=? and disguise_type=?";
    let disguise: DisguiseFromDB = sqlx::query_as(sql)
        .bind(vault_id)
        .bind(name)
        .fetch_one(vault_pool)
        .await?;
    //get the decorrelated publications' predicate
    let sql = "SELECT * FROM function WHERE disguise_id=? and function_type=?";
    let functions: Vec<Function> = sqlx::query_as(sql)
        .bind(disguise.disguise_id)
        .bind("decorrelation")
        .fetch_all(vault_pool)
        .await?;
    //destroy them
    for function in functions {
        delete_decorrelated_targets_db(
            target_pool,
            function.table_name.unwrap().as_str(),
            function.predicate.unwrap().as_str()
        ).await?;
    }
    //delete functions by disguise id
    let sql = "DELETE FROM function WHERE disguise_id=?";
    sqlx::query(sql)
        .bind(disguise.disguise_id)
        .execute(vault_pool)
        .await?;
    //delete the disguise
    let sql = "DELETE FROM disguise WHERE vault_id=? and disguise_type=?";
    sqlx::query(sql)
        .bind(vault_id)
        .bind(name)
        .execute(vault_pool)
        .await?;
    Ok("The old data in vault has been deleted.".to_string())
}



#[cfg(test)]
mod tests {
    use std::env;
    use std::sync::Mutex;
    use actix_web::http::StatusCode;
    use actix_web::web;
    use sqlx::mysql::MySqlPoolOptions;
    use dotenv::dotenv;
    use crate::dbaccess::vault::{generate_vault_db, get_vault_by_id_db, upload_disguise_object_db};
    use crate::handlers::vault::generate_vault;
    use crate::models::placeholder::{GeneratePlaceHolder, PlaceholderInfo};
    use crate::models::vault::{Disguise, Function, GenerateVault, Vault};
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
        //put the target and vault database pool in the state
        let shared_data = web::Data::new(AppState {
            vault_db,
            target_db,
        });
        let placeholder_info = PlaceholderInfo {
            pred: Some("contact_id=0".into()),
            table: Some("contact_info".into()),
        };
        let vault = Vault {
            vault_id: Some("19".into()),
            email: Some("bea@mail.com".into()),
            placeholder_info: Some(serde_json::to_string(&placeholder_info).unwrap()),
        };
        let res = generate_vault_db(&shared_data.vault_db, vault).await;
        assert_eq!(res.is_ok(), true);
    }

    #[ignore]
    #[actix_rt::test]
    async fn get_vault_by_id_db_test() {
        //load the env variables
        dotenv().ok();
        let vault_database_url = env::var("VAULT_DATABASE_URL").expect("VAULT_DATABASE_URL is not set yet.");
        let vault_db = MySqlPoolOptions::new().connect(&vault_database_url).await.unwrap();
        let res = get_vault_by_id_db(&vault_db, "19").await;
        assert_eq!(res.is_ok(), true);
    }

    #[ignore]
    #[actix_rt::test]
    async fn upload_disguise_object_db_test() {
        //load the env variables
        dotenv().ok();
        let vault_database_url = env::var("VAULT_DATABASE_URL").expect("VAULT_DATABASE_URL is not set yet.");
        let vault_db = MySqlPoolOptions::new().connect(&vault_database_url).await.unwrap();
        let function = Function {
            disguise_id: Some(999),
            function_type: Some("sdf".to_string()),
            table_name: Some("fsdf".to_string()),
            predicate: Some("sf".to_string()),
            original: Some("sf".to_string()),
            updated: Some("sf".to_string())
        };
        let disguise = Disguise {
            disguise_id: Some(999),
            time: Some("1111".to_string()),
            vault_id: Some("19".to_string()),
            disguise_type: Some("123".to_string()),
            functions: Some(vec![function])
        };
        let res = upload_disguise_object_db(&vault_db, &disguise).await;
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
    }


}