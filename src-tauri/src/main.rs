#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

const DATABASE_NAME: &str = "datent";
const COLLECTION_NAME: &str = "kasane";

use mongodb::{bson::{doc, Document}, options::{ClientOptions}, Client, Database};
use once_cell::sync::OnceCell;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{anyhow, Error};

static DB: OnceCell<Database> = OnceCell::new();

#[tauri::command]
async fn db_connect() -> tauri::Result<bool> {
  if DB.get().is_none() == false {
    return Ok(false);
  }

  let attempt = connect_client().await;
  match attempt {
    Ok(client) => {
      let _set_db = DB.set(client.database(DATABASE_NAME)).unwrap();
      return Ok(true);
    },
    Err(msg) => {
      println!("{}", msg);
      return Err(tauri::Error::CreateWindow);
    },
  }
}

async fn connect_client() -> Result<Client, Error> {
  let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
  client_options.app_name = Some("CDE".to_string());
  let client = Client::with_options(client_options)?;

  // Attempt connection
  match client.database("admin").run_command(doc! {"ping": 1}, None).await {
    Ok(_) => {
      println!("Connected to database.");
      return Ok(client);
    },
    _ => {
      return Err(anyhow!("Could not connect to database."));
    }
  };
}

#[allow(dead_code)]
fn print_type_of<T>(_: &T) {
  println!("{}", std::any::type_name::<T>())
}

#[tauri::command]
async fn insert_dummytest() {
  let start = SystemTime::now();
  let since_the_epoch = start
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards");

  let db = DB.get().unwrap();
  let collection = db.collection::<Document>("dummytest");
  let doc = doc! { "time": since_the_epoch.as_secs() as u32 };
  collection.insert_one(doc, None).await.unwrap();
}

#[tauri::command]
async fn get_all_documents() -> Vec<Document> {
  if DB.get().is_none() { return vec![]; }

  let db = DB.get().unwrap();
  let collection = db.collection::<Document>(COLLECTION_NAME);

  let mut documents: Vec<Document> = vec![];
  let mut cursor = collection.find(None, None).await.unwrap();
  while cursor.advance().await.unwrap() {
    documents.push(cursor.deserialize_current().unwrap());
  }
  return documents;
}

#[tokio::main]
async fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![insert_dummytest, get_all_documents, db_connect])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
