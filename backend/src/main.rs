use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::http::header::{ContentDisposition, DispositionType};
use actix_web::{middleware, web, App, HttpResponse, HttpServer, get, HttpRequest, Error};
use actix_web_static_files::ResourceFiles;
use chrono:: Utc;
use serde::{Deserialize, Serialize};
mod shared_structs;
use itertools::Itertools;
use mongodb::bson::doc;
use mongodb::{options::ClientOptions, Client, Collection};
use shared_structs::*;

async fn create_sheet(
    sheet_collection: web::Data<Collection<Sheet>>,
    req: web::Json<CreateSheetRequest>,
) -> HttpResponse {
    let req = req.0;
    log::info!("Received request to create sheet {:?}", req);
    if req.sheet_name.len() > 50 {
        return HttpResponse::NotAcceptable().body("Group name exceeds maximum length".to_owned());
    }
    if req.sheet_name.is_empty() {
        return HttpResponse::NotAcceptable().body("Cannot have empty group name".to_owned());
    }
    let mut sheet_uuid = String::from(url_escape::encode_fragment(&req.sheet_name));
    sheet_uuid.push('-');
    let raw_uuid = uuid::Uuid::new_v4().to_string();
    sheet_uuid.push_str(&raw_uuid);

    let sheet = Sheet {
        uuid: sheet_uuid.clone(),
        name: req.sheet_name,
        version_number: 0,
        members: HashMap::new(),
        transactions: HashMap::new(),
    };
    if sheet_collection.insert_one(sheet, None).await.is_ok() {
        let response = CreateSheetResponse { sheet_uuid };
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::InternalServerError()
            .body("Error Processing Request. Please try again".to_owned())
    }
}

async fn get_sheet(
    sheet_collection: web::Data<Collection<Sheet>>,
    req: web::Json<GetSheetRequest>,
) -> HttpResponse {
    let req = req.0;
    log::info!("Received request to get sheet {:?}", req);
    match sheet_collection
        .find_one(doc! { "uuid": &req.sheet_uuid }, None)
        .await
    {
        Ok(Some(sheet)) => HttpResponse::Ok().json(sheet),
        Ok(None) => HttpResponse::NotFound().body("Sheet not found! Check your link"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
async fn fork_sheet(
    sheet_collection: web::Data<Collection<Sheet>>,
    req: web::Json<ForkSheetRequest>,
) -> HttpResponse {
    let req = req.0;
    log::info!("Received request to get sheet {:?}", req);

    let old_sheet = match sheet_collection
        .find_one(doc! { "uuid": &req.sheet_uuid }, None)
        .await
    {
        Ok(Some(sheet)) => sheet,
        Ok(None) => {
            return HttpResponse::NotFound().body("Sheet not found! This should not be possible")
        }
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };
    let mut new_sheet_uuid = String::from(url_escape::encode_fragment(&old_sheet.name));
    new_sheet_uuid.push('-');
    let raw_uuid = uuid::Uuid::new_v4().to_string();
    new_sheet_uuid.push_str(&raw_uuid);
    let mut new_sheet = old_sheet.clone();
    new_sheet.uuid = new_sheet_uuid.clone();

    if sheet_collection.insert_one(new_sheet, None).await.is_ok() {
        let response = ForkSheetResponse {
            sheet_uuid: new_sheet_uuid,
        };
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::InternalServerError()
            .body("Error Processing Request. Please try again".to_owned())
    }
}
async fn update_sheet(
    sheet_collection: web::Data<Collection<Sheet>>,
    req: web::Json<UpdateSheetRequest>,
) -> HttpResponse {
    let req = req.0.sheet;
    log::info!("Received request to update sheet {:?}", &req.uuid);
    for transaction in req.transactions.values() {
        if transaction.name.is_empty() || transaction.name.len() > 100 {
            return HttpResponse::NotAcceptable().body("Invalid transaction name");
        }

        if transaction.giver_uuid.is_empty() {
            return HttpResponse::NotAcceptable()
                .body("Must specify a valid person to have paid for the transaction");
        }
        if !req.members.contains_key(&transaction.giver_uuid) {
            return HttpResponse::NotAcceptable()
                .body("Unable to save. You cannot remove a member who is part of a transaction.");
        }
        for uuid in transaction.line_items.keys() {
            if !req.members.contains_key(uuid) {
                return HttpResponse::NotAcceptable().body(
                    "Unable to save. You cannot remove a member who is part of a transaction",
                );
            }
        }
        if !req.members.contains_key(&transaction.giver_uuid) {
            return HttpResponse::NotAcceptable()
                .body("Must specify a valid person to have paid for the transaction.");
        }
        if transaction.line_items.is_empty() {
            return HttpResponse::NotAcceptable().body("Transaction does not balance. Please specify who is responsible for reimbursing for this transaction");
        }

        let mut calculated_sum = 0.;
        for li in transaction.line_items.values() {
            if let Some(x) = li.locked_payment_amount {
                calculated_sum += x;
            } else {
                calculated_sum = transaction.transaction_amount;
                break;
            }
        }
        let diff: f64 = (transaction.transaction_amount - calculated_sum).abs();
        if diff > 0.1 {
            return HttpResponse::NotAcceptable().body("Transaction does not balance. Try checking the sums of the line items or having at least one unlocked line item");
        }
    }
    if req
        .members
        .values()
        .map(|p| p.name.clone())
        .duplicates()
        .next()
        .is_some()
    {
        return HttpResponse::NotAcceptable().body("Duplicate names for members of the group.");
    }
    match sheet_collection
        .find_one_and_replace(doc! { "uuid": &req.uuid }, req.clone(), None)
        .await
    {
        Ok(Some(_)) => HttpResponse::Ok().json(req),
        Ok(None) => HttpResponse::NotFound().body("Sheet not found! Check your link"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
#[derive(Deserialize, Debug, PartialEq)]
struct LoginConfig {
    user: String,
    password: String,
    url: String,
}
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server ");
    let mut config_file = dirs::home_dir().unwrap();
    config_file.push("Keys/BalanceSheetMdb.json5");
    let login_info = fs::read_to_string(config_file)?;
    let login_info = json5::from_str::<LoginConfig>(&login_info).unwrap();

    let connection_string = format!(
        "mongodb+srv://{}:{}@{}/?retryWrites=true&w=majority",
        login_info.user, login_info.password, login_info.url
    );
    let client_options = ClientOptions::parse(connection_string).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let database = client.database("BalanceTracker");
    let sheets = database.collection::<Sheet>("sheets");


    let frontend= actix_web::rt::spawn(async move {
        HttpServer::new(move || {
            let generated = generate();
            App::new().service(ResourceFiles::new("/", generated))
        })
        .bind("127.0.0.1:8080")?
        .workers(1)
        .run()
        .await
    });
    let backend= HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(sheets.clone()))
            .app_data(web::JsonConfig::default().limit(40000096)) // <- limit size of the payload (global configuration)
            .service(web::resource("/createsheet").route(web::post().to(create_sheet)))
            .service(web::resource("/getsheet").route(web::post().to(get_sheet)))
            .service(web::resource("/updatesheet").route(web::post().to(update_sheet)))
            .service(web::resource("/forksheet").route(web::post().to(fork_sheet)))
    })
    .workers(1)
    .bind(("127.0.0.1", 7525))?
    .run();
    futures::future::join(frontend,backend).await.0.unwrap()
    // frontend.await;
    // backend.await
}

