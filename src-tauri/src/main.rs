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
use serde_json::Value;
use bson::oid::ObjectId;

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

#[allow(unreachable_code)]
#[tauri::command]
async fn save_document(id: Value, tsu: Value, tsa: Value, the_type: Value, note: Value, volume: Value, page: Value) -> tauri::Result<bool> {
	let start = SystemTime::now();
	let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();

	let db = DB.get().unwrap();
	let collection = db.collection::<Document>(COLLECTION_NAME);
	
	let mut doc: Document;
	let mut undoc: Document = doc!{};
	let oid: ObjectId;

	if id.is_null() {
		doc = doc!{ "created": since_the_epoch.as_secs() as u32 };
	} else {
		doc = doc!{ "updated": since_the_epoch.as_secs() as u32 };
	}

	if !tsu.is_null() { doc.insert("tsu", tsu.as_str()); }
	if !tsa.is_null() { doc.insert("tsa", tsa.as_str()); }
	if !the_type.is_null() { doc.insert("type", the_type.as_str()); }
	if !note.is_null() { doc.insert("note", note.as_str()); }
	if !volume.is_null() { doc.insert("volume", volume.as_str()); }
	if !page.is_null() { doc.insert("page", page.as_str()); }

	if !id.is_null() {
		if tsu.is_null() { undoc.insert("tsu", ""); }
		if tsa.is_null() { undoc.insert("tsa", ""); }
		if the_type.is_null() { doc.insert("type", ""); }
		if note.is_null() { undoc.insert("note", ""); }
		if volume.is_null() { undoc.insert("volume", ""); }
		if page.is_null() { undoc.insert("page", ""); }
	}
	
	if id.is_null() {
		collection.insert_one(doc, None).await.unwrap();
		return Ok(true);
	} else {
		let mut setdoc = doc!{};
		if doc.len() > 0 { setdoc.insert("$set", doc); }
		if undoc.len() > 0 { setdoc.insert("$unset", undoc); }
		oid = ObjectId::parse_str(id.as_str().unwrap()).unwrap();
		collection.find_one_and_update(doc!{ "_id": oid, }, setdoc, None).await.unwrap();
		return Ok(true);
	}
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
		.invoke_handler(tauri::generate_handler![save_document, get_all_documents, db_connect])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
