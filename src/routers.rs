use actix_web::*;
use crate::handlers::disguise::*;
use crate::handlers::vault::generate_vault;

/// all the vault interfaces
pub fn vault_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/vault")
        .route("/generate", web::post().to(generate_vault)));
}

/// all the disguise interfaces
pub fn disguise_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/disguise")
        .route("/userscrub", web::post().to(scrub_user))
        .route("/anonymize", web::post().to(anonymize))
        .route("/expiration", web::post().to(expiration))
        .route("/clearvault", web::post().to(clear_vault))
        .route("/recover", web::post().to(recover_disguise)));
}
