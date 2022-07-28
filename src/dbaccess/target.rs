use chrono::{DateTime, Duration, Local};
use sqlx::{MySqlPool, Row};
use crate::error::MyError;
use crate::models::placeholder::GeneratePlaceHolder;
use crate::models::target::{Field, Target};
use crate::models::transformation::Transformation;
use crate::models::vault::Disguise;

///
/// generate the placeholder in the target database
/// (the placeholder could be generated only once)
///
/// # Arguments
///
/// * `target_db_pool`: the web-application's database
/// * `placeholder`: the parameter sent by user or app
///
/// returns: Result<String, MyError>
///
pub async fn generate_placeholder_db(
    target_db_pool: &MySqlPool,
    placeholder: &GeneratePlaceHolder,
) -> Result<String, MyError> {
    //insert the generate placeholder into database
    let mut sql = String::from("");
    sql.push_str("INSERT INTO ");
    sql.push_str(placeholder.table.as_ref().unwrap().as_str());
    sql.push_str("(");
    sql.push_str(placeholder.fields.as_ref().unwrap().as_str());
    sql.push_str(") VALUES(");
    sql.push_str(placeholder.field_values.as_ref().unwrap().as_str());
    sql.push_str(")");
    let id = sqlx::query(sql.as_str())
        .execute(target_db_pool)
        .await?
        .last_insert_id();

    //the operation is successful
    //return the id of the new generated placeholder
    Ok(id.to_string())
}

///
/// get all the operated targets from the target_pool database
/// there are two layers of vector,
/// because one transformation might operate several targets
///
/// # Arguments
///
/// * `target_pool`: the application's database
/// * `transformations`: the transformations of the disguise
///
/// returns: Result<Vec<Vec<Target, Global>, Global>, MyError>
///
pub async fn get_targets_db(
    target_pool: &MySqlPool,
    transformations: &Vec<Transformation>
) -> Result<Vec<Vec<Target>>, MyError> {
    let mut res = vec![];

    //every transformation
    for transformation in transformations {
        //every row affected by the same transformation
        let table_name = transformation.table_name.as_ref().clone().unwrap().as_str();
        let predicate = transformation.predicate.as_ref().clone().unwrap().as_str();
        let mut targets = vec![];
        //get the names and types of target's fields in this table
        //and the primary key's index
        let mut field_names = vec![];
        let mut field_types = vec![];
        let mut primary_key_index = 0;
        let sql = "DESC ".to_string() + table_name;
        let rows = sqlx::query(sql.as_str())
            .fetch_all(target_pool)
            .await?;
        for (i, row) in rows.iter().enumerate() {
            let field_name: String = row.get("Field");
            let field_type: String = row.get("Type");
            let primary_key: String = row.get("Key");
            field_names.push(field_name);
            field_types.push(field_type);
            if primary_key.eq("PRI") {
                primary_key_index = i;
            }
        }
        //query the targets
        let sql = "SELECT * FROM ".to_string() + table_name + " WHERE " + predicate;
        let query_res = sqlx::query(sql.as_str())
            .fetch_all(target_pool)
            .await;
        match query_res {
            //push every target into the targets vector
            Ok(rows) => {
                //if the query result is 0
                if rows.len() == 0 {
                    return Err(MyError::OperationError("No data fits the requirement.".to_string()));
                }
                for row in rows {
                    let mut fields = vec![];
                    //push every field of the row(target)
                    for (i, field_name) in field_names.iter().enumerate() {
                        let field_type = &field_types[i];
                        let mut field_value = String::new();
                        if field_type.contains("varchar") {
                            let value: String = row.get(field_name.as_str());
                            field_value.push_str(value.as_str());
                        }else if field_type.contains("int") {
                            let value: i32 = row.get(field_name.as_str());
                            field_value.push_str(value.to_string().as_str());
                        }else if field_type.contains("time"){
                            let value: DateTime<Local> = row.get(field_name.as_str());
                            field_value.push_str(value.to_string().as_str());
                        }
                        fields.push(Field {
                            field_name: Some(field_name.clone()),
                            field_type: Some(field_type.clone()),
                            field_value: Some(field_value)
                        })
                    }
                    targets.push(Target {
                        primary_key_index: Some(primary_key_index),
                        fields: Some(fields)
                    })
                }
                res.push(targets);

            }
            //the input err make the query failed.
            Err(_) => {
                return Err(MyError::InvalidInput("The input \"table\" or \"predicate\" is not correct.".to_string()));
            }
        };
    }
    Ok(res)
}

///
/// execute the transformations to the target database
/// return the changes of the transformations
///
/// # Arguments
///
/// * `placeholder_pred`: the placeholder's predicate
/// * `target_pool`: application's database
/// * `transformations`: the transformations of the disguise
///
/// returns: Result<Vec<String, Global>, MyError>
///
pub async fn execute_transformations_db(
    placeholder_pred: &str,
    target_pool: &MySqlPool,
    transformations: &Vec<Transformation>
) -> Result<Vec<String>, MyError> {
    //to store all the changes in transformations
    let mut all_changes = vec![];
    //execute the transformations
    for transformation in transformations {
        //get the table_name and predicate
        let table_name = transformation.table_name.as_ref().unwrap();
        let predicate = transformation.predicate.as_ref().unwrap();
        //check the transformations' type
        match transformation.transform_type.as_ref().unwrap().to_lowercase().as_str() {
            //if the transformation is removal
            "removal" => {
                //generate the sql sentence
                let mut sql = String::from("");
                sql.push_str("DELETE FROM ");
                sql.push_str(&table_name);
                sql.push_str(" WHERE ");
                sql.push_str(&predicate);
                //execute the sql
                let num = sqlx::query(sql.as_str())
                    .execute(target_pool)
                    .await?
                    .rows_affected();
                //if the predicate is not correct
                if num <= 0 {
                    return Err(MyError::InvalidInput("The predicate is not correct.".to_string()));
                }
                all_changes.push("".to_string());
            }
            //if the transformation is modification
            "modification" => {
                let changes = transformation.changes.as_ref().unwrap().as_str();
                //generate the sql sentence
                let mut sql = String::from("");
                sql.push_str("UPDATE ");
                sql.push_str(&table_name);
                sql.push_str(" SET ");
                sql.push_str(changes);
                sql.push_str(" WHERE ");
                sql.push_str(&predicate);
                //execute the sql
                let num = sqlx::query(
                    sql.as_str()
                )
                    .execute(target_pool)
                    .await?
                    .rows_affected();
                //if the predicate is not correct
                if num <= 0 {
                    return Err(MyError::InvalidInput("The predicate is not correct.".to_string()));
                }
                all_changes.push(String::from(changes));
            }
            //if the transformation is decorrelation
            "decorrelation" => {
                // let foreign_key = transformation.foreign_key.unwrap();
                let changes = placeholder_pred;

                //generate the sql sentence
                let mut sql = String::from("");
                sql.push_str("UPDATE ");
                sql.push_str(&table_name);
                sql.push_str(" SET ");
                sql.push_str(changes);
                sql.push_str(" WHERE ");
                sql.push_str(&predicate);
                //execute the sql
                let num = sqlx::query(
                    sql.as_str()
                )
                    .execute(target_pool)
                    .await?
                    .rows_affected();
                //if the predicate is not correct
                if num <= 0 {
                    return Err(MyError::InvalidInput("The predicate is not correct.".to_string()));
                }
                all_changes.push(String::from(changes));
            }
            //if the transform type is different from the three types
            _ => {
                return Err(MyError::InvalidInput(
                    "The transform type is not correct, \
                    the execution has been terminated.".to_string()
                ));
            }
        };
    }
    Ok(all_changes)
}

///
/// recover the applied disguise in the vault
/// the disguises can be downloaded by the function "download_disguise_db"
///
/// # Arguments
///
/// * `target_db`: the application's database
/// * `disguise`: the information of the disguise from vault
///
/// returns: Result<String, MyError>
///
pub async fn recover_db(
    target_db: &MySqlPool,
    disguise: &Disguise,
) -> Result<String, MyError>{
    let mut functions = disguise.functions.as_ref().unwrap().clone();
    functions.reverse();

    for function in functions {
        let function_type = function.function_type.as_ref().unwrap();
        let table_name = function.table_name.as_ref().unwrap();
        let predicate = function.predicate.as_ref().unwrap();
        let original = function.original.as_ref().unwrap();

        match function_type.as_str() {
            //if the function type is removal
            "removal" => {
                let sql = "INSERT INTO ".to_string() + table_name + " VALUES (" + original + ")";
                sqlx::query(sql.as_str())
                    .execute(target_db)
                    .await?;
            },
            //if the function type is modification or decorrelation
            _ => {
                //get the fields name
                let mut fields = vec![];
                let sql = "DESC ".to_string() + table_name;
                let rows = sqlx::query(sql.as_str())
                    .fetch_all(target_db)
                    .await?;
                for row in rows {
                    let field: &str = row.get(0);
                    fields.push(String::from(field));
                }
                //transform to the format of "field_name=field_value"
                let values = original.split(", ").collect::<Vec<&str>>();
                let mut updates = String::new();
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        updates += ",";
                    }
                    updates = updates + field + "=" + values.get(i).as_ref().unwrap();
                }
                //update the values
                let sql = "UPDATE ".to_string() + table_name + " SET " + updates.as_str() + " WHERE " + predicate;
                sqlx::query(sql.as_str())
                    .execute(target_db)
                    .await?;
            }
        }
    }
    Ok("The target has been recovered.".to_string())
}

///
/// some transformations' predicate is not the primary key
/// (such as "time<2022-7-14")
/// this function could transfer them into primary key predicate
/// the old transformation could be transferred into more than one new transformations
/// the transformations of user table will be placed at the end one by one
/// and the other tables' transformations will be placed at the front
/// in case causing dangling references
///
/// # Arguments
///
/// * `target_pool`: application's database
/// * `transformations`: the transformations whose predicate is not primary key
/// * `delete_age`: the number of years who is inactive will be deleted
///
/// returns: Result<Vec<Transformation, Global>, MyError>
///
pub async fn transfer_transformations(
    target_pool: &MySqlPool,
    transformations: &Vec<Transformation>,
    delete_age: i64
) -> Result<Vec<Transformation>, MyError> {
    let mut res = vec![];
    let mut users_transformation = Transformation::new_empty();
    let mut user_transformations = vec![];
    let mut contribution_transformations = vec![];
    let delete_time = Local::now() - Duration::days(365 * delete_age);

    //transform the predicate into "last_login_time<delete_time"
    for transformation in transformations {
        //the transformation is for users
        if transformation.predicate.is_some() {
            users_transformation = transformation.clone();
            let predicate = users_transformation.predicate.as_ref().unwrap().clone();
            users_transformation.predicate = Some(predicate + "<\"" + delete_time.to_string().as_str() + "\"");
        }else {
            //the transformation is for contributions
            contribution_transformations.push(transformation.clone());
        }
    }

    //get the users who didn't login for a long time
    let targets = get_targets_db(target_pool, &vec![users_transformation.clone()]).await?;
    //transformed contribution transformations
    let mut new_contribution_transformations = vec![];
    for target in &targets[0] {
        let primary_key = target.primary_key().unwrap();
        let predicate = primary_key.field_name.unwrap().clone() + "=" + primary_key.field_value.unwrap().as_str();
        let mut user_transformation = users_transformation.clone();
        user_transformation.predicate = Some(predicate);

        for transformation in &contribution_transformations {
            let foreign_key = target.foreign_key(transformation.foreign_key.as_ref().unwrap().as_str()).unwrap();
            let predicate = foreign_key.field_name.unwrap() + "=" + foreign_key.field_value.unwrap().as_str();
            new_contribution_transformations.push(Transformation {
                transform_type: transformation.transform_type.clone(),
                table_name: transformation.table_name.clone(),
                predicate: Some(predicate),
                foreign_key: transformation.foreign_key.clone(),
                changes: None
            });
        }

        user_transformations.push(user_transformation);
    }

    for transformation in new_contribution_transformations {
        res.push(transformation);
    }
    for transformation in user_transformations {
        res.push(transformation);
    }

    Ok(res)
}

///
/// delete the decorrelated targets
/// when the disguises in vault are deleted
///
/// # Arguments
///
/// * `target_pool`: application's database
/// * `table_name`: the table name of the publications
/// * `predicate`: the predicate of the publications
///
/// returns: Result<String, MyError>
///
pub async fn delete_decorrelated_targets_db(
    target_pool: &MySqlPool,
    table_name: &str,
    predicate: &str
) -> Result<String, MyError> {
    let sql = "DELETE FROM ".to_string() + table_name + " WHERE " + predicate;
    sqlx::query(sql.as_str())
        .execute(target_pool)
        .await?;
    Ok("The decorrelated targets has been deleted.".to_string())
}

//get the state of the objects before or after the transformations
// pub async fn get_targets_db(
//     target_pool: &MySqlPool,
//     transformations: &Vec<Transformation>
// ) -> Result<Vec<Vec<String>>, MyError> {
//     let mut res = vec![];
//     //iterate all the transformations
//     //log all the fields' value of the affected data
//     //put them into res
//     for transformation in transformations {
//         let table = transformation.table_name.as_ref().clone().unwrap().as_str();
//         let predicate = transformation.predicate.as_ref().clone().unwrap().as_str();
//         let mut all_values = vec![];
//         //get the affected data
//         let sql = "SELECT * FROM ".to_string() + table + " WHERE " + predicate;
//         let rows = sqlx::query(sql.as_str())
//             .fetch_all(target_pool)
//             .await;
//         //check if the data is exist
//         match rows {
//             //not exist
//             //means the data has been "removed"
//             //push an empty vec
//             Err(_) => {
//                 res.push(vec![]);
//             }
//             //put the data's fields' value into res as string
//             Ok(_) => {
//                 //iterate all rows affected
//                 for row in rows.unwrap() {
//                     let len = row.len();
//                     let mut values = String::new();
//                     let mut i = 0;
//                     //iterate all values in one row
//                     while i < len {
//                         //get the value as string
//                         let temp_res: Result<&str, MyError> = row.try_get(i).map_err(|_err| MyError::DBError("The got type is not string.".to_string()));
//                         let mut value = String::new();
//                         match temp_res {
//                             //if the type of value is not string
//                             //covert the type into number
//                             Err(_err) => {
//                                 let a: i32 = row.get(i);
//                                 value += a.to_string().as_str();
//                             }
//                             //if the type is string
//                             //covert the the value into the format of "value"
//                             _ => {
//                                 value += "\"";
//                                 value += temp_res.unwrap();
//                                 value += "\"";
//                             }
//                         }
//                         //separate values with ", "
//                         if i != 0 {
//                             values += ", ";
//                         }
//                         //add the "value" to the "values"
//                         values += value.as_str();
//                         i += 1;
//                     }
//                     all_values.push(values);
//                 }
//             }
//         };
//         res.push(all_values);
//     }
//     Ok(res)
// }

//
// pub async fn get_primary_keys_db(
//     target_db: &MySqlPool,
//     transformations: &Vec<Transformation>
// ) -> Result<Vec<Vec<String>>, MyError> {
//     let mut keys_vec = vec![];
//
//     for transformation in transformations {
//         let table_name = transformation.table_name.as_ref().unwrap();
//         let predicate = transformation.predicate.as_ref().unwrap();
//         //get the key's name
//         let mut keys = vec![];
//         let sql = "SELECT column_name FROM INFORMATION_SCHEMA.`KEY_COLUMN_USAGE` WHERE table_name=? AND constraint_name='PRIMARY'";
//         let key_name = sqlx::query(sql)
//             .bind(transformation.table_name.as_ref().unwrap())
//             .fetch_one(target_db)
//             .await?;
//         let key_name: String = key_name.get("column_name");
//
//         //get keys' values of the same transformation
//         let sql = "SELECT ".to_string() + key_name.as_str() + " FROM " + table_name + " WHERE " + predicate;
//         let rows = sqlx::query(sql.as_str())
//             .fetch_all(target_db)
//             .await?;
//         for row in rows {
//             let key_value:i32 = row.get(key_name.as_str());
//             let key_predicate = String::from(key_name.as_str()) + "=" + key_value.to_string().as_str();
//             keys.push(key_predicate);
//         }
//         keys_vec.push(keys);
//     }
//
//     Ok(keys_vec)
// }