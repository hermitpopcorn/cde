#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

const DATABASE_NAME: &str = "datent";
const COLLECTION_NAME: &str = "kasane";

use tauri::{CustomMenuItem, Menu, WindowMenuEvent};
use mongodb::{bson::{doc, Document}, options::{ClientOptions, FindOptions, Collation}, Client, Database};
use once_cell::sync::OnceCell;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{anyhow, Error};
use bson::oid::ObjectId;
use maplit::hashmap;

static DB: OnceCell<Database> = OnceCell::new();


#[derive(Clone, serde::Serialize)]
struct ChangePagePayload {
	page: String,
}

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
			eprintln!("{}", msg);
			return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Could not connect to Database."))));
		},
	}
}

async fn connect_client() -> Result<Client, Error> {
	let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
	client_options.app_name = Some("CDE".to_string());
	let client = Client::with_options(client_options)?;

	// attempt connection
	match client.database("admin").run_command(doc!{"ping": 1}, None).await {
		Ok(_) => Ok(client),
		_ => Err(anyhow!("Could not connect to database.")),
	}
}

#[tauri::command]
async fn save_document(
	id: Option<&str>, tsu: Option<&str>, tsa: Option<&str>,
	the_type: Option<&str>, note: Option<&str>, volume: Option<&str>, page: Option<&str>,
	cause: Option<&str>, effects: Vec<&str>
) -> tauri::Result<()> {
	// generate timestamp
	let start = SystemTime::now();
	let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("duration since the epoch");

	let db = DB.get().expect("db connection");
	let collection = db.collection::<Document>(COLLECTION_NAME);
	
	let mut doc: Document;
	let mut undoc: Document = doc!{};

	// fill created or updated depending on operation (insert/update)
	match id {
		Some(_) => {
			doc = doc!{ "created": since_the_epoch.as_secs() as u32 };
		},
		None => {
			doc = doc!{ "updated": since_the_epoch.as_secs() as u32 };
		},
	};

	let fields = hashmap!{
		"tsu" => &tsu,
		"tsa" => &tsa,
		"type" => &the_type,
		"note" => &note,
		"volume" => &volume,
		"page" => &page,
		"cause" => &cause,
	};
	for (&field_name, variable_ref) in &fields {
		match variable_ref {
			Some(the_string) => {
				doc.insert(field_name, the_string);
			},
			None => {
				// if field value is empty and this is an update opertaion, unset
				if id != None {
					undoc.insert(field_name, "");
				};
			},
		};
	}

	if effects.len() > 0 {
		doc.insert("effects", effects);
	} else {
		if id != None { undoc.insert("effects", ""); }
	}
	
	if id == None {
		// insert operation
		let result = collection.insert_one(doc, None).await;
		if result.is_err() { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Insert error.")))); }
	} else {
		// update operation
		// create container document
		let mut setdoc = doc!{};
		if doc.len() > 0 { setdoc.insert("$set", doc); }
		if undoc.len() > 0 { setdoc.insert("$unset", undoc); }

		let result = collection.find_one_and_update(doc!{ "_id": ObjectId::parse_str(id.unwrap()).expect("valid oid"), }, setdoc, None).await;
		if result.is_err() { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Update error.")))); }
		if result.unwrap().is_none() { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Did not update any data. Maybe ID is specified despite not existing in database?")))); }
	}

	Ok(())
}

#[tauri::command]
async fn remove_document(id: Option<&str>) -> tauri::Result<()> {
	let db = DB.get().expect("db connection");
	let collection = db.collection::<Document>(COLLECTION_NAME);

	let oid: ObjectId = ObjectId::parse_str(id.unwrap()).unwrap();
	let result = collection.delete_one(doc!{ "_id": oid }, None).await;
	if result.is_err() { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Delete error.")))); }
	Ok(())
}

#[tauri::command]
async fn get_documents(page: Option<u64>, size: Option<u64>, filters: Option<std::collections::HashMap<String, String>>) -> (Vec<Document>, u64, u64) {
	if DB.get().is_none() { return (vec![], 1, 0); }

	let db = DB.get().expect("db connection");
	let collection = db.collection::<Document>(COLLECTION_NAME);

	// construct search criteria document
	let mut search: Option<Document> = None;
	if !filters.is_none() {
		let mut filter_docs: Vec<Document> = vec![];
		for (key, value) in filters.unwrap().into_iter() {
			if value.len() < 1 { continue; }

			match key.as_str() {
				"volume" => {
					// match exactly
					filter_docs.push(doc!{ key: value });
				},
				"page" => {
					// get if starting matches
					filter_docs.push(doc!{ key: bson::Regex{ pattern: "^".to_owned() + regex::escape(&value).as_str(), options: String::from("i") } });
				},
				"type" => {
					// match exactly
					filter_docs.push(doc!{ key: value });
				},
				"note" => {
					// get if contains OR doesn't contain
					match &value.starts_with("-") {
						false => {
							filter_docs.push(doc!{ key: bson::Regex{ pattern: regex::escape(&value), options: String::from("im") } });
						},
						true => {
							let mut new_value = String::from(&value);
							new_value.remove(0);
							filter_docs.push(doc!{ key: doc!{ "$not": bson::Regex{ pattern: regex::escape(&new_value), options: String::from("im") } } });
						},
					}
				},
				"text" => {
					// get if contains in tsa or tsu
					filter_docs.push(doc!{ "$or": [doc!{ "tsa": bson::Regex{ pattern: regex::escape(&value), options: String::from("im") } }, doc!{ "tsu": bson::Regex{ pattern: regex::escape(&value), options: String::from("im") } }] });
				},
				default => {
					eprintln!("found unknown filter key {}", default);
				},
			}
		}

		if filter_docs.len() > 0 {
			search = Some(doc!{ "$and": filter_docs });
		}
	}

	let mut page: u64 = page.unwrap_or(1);
	let size: u64 = size.unwrap_or(20);
	
	let total_filtered_count: u64 = collection.count_documents(search.clone(), None).await.expect("filtered document count");
	let mut skip: u64 = (page - 1) * size;
	while skip > total_filtered_count {
		page -= 1;
		skip = (page - 1) * size;
	}

	let sort = doc!{ "volume": 1, "page": 1, "created": 1 };
	let collation = Collation::builder().locale("en_US").numeric_ordering(true).build();

	let mut documents: Vec<Document> = vec![];
	let mut cursor = collection.find(search, FindOptions::builder().sort(sort).collation(collation).skip(skip).limit(size as i64).build()).await.unwrap();
	while cursor.advance().await.unwrap() {
		documents.push(cursor.deserialize_current().unwrap());
	}

	(documents, page, total_filtered_count)
}

#[tauri::command]
async fn star_document(id: &str) -> tauri::Result<bool> {
	let db = DB.get().expect("db connection");
	let collection = db.collection::<Document>(COLLECTION_NAME);

	let oid: ObjectId = ObjectId::parse_str(id).expect("valid oid");
	let search = collection.find_one(doc!{ "_id": oid }, None).await.unwrap();

	if search.is_none() {
		return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Document not found."))));
	}

	let doc: Document = search.unwrap();
	let starred: bool = doc.get_bool("starred").unwrap_or(false);

	let setdoc = if !starred { doc!{ "$set": doc!{ "starred": true } } } else { doc!{ "$unset": doc!{ "starred": false } } };
	let result = collection.find_one_and_update(doc!{ "_id": oid, }, setdoc, None).await;
	if result.is_err() { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Update error.")))); }
	if result.unwrap().is_none() { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Did not update any data. Maybe data does not exist in database?")))); }
	
	Ok(!starred)
}

#[derive(serde::Serialize)]
struct AggregateCount {
	columns: Vec<(String, String)>,
	quantity: u32,
}

#[tauri::command]
async fn get_count_by_columns(columns: Vec<&str>) -> tauri::Result<Vec<AggregateCount>> {
	if DB.get().is_none() { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Not connected to database.")))); }

	let db = DB.get().expect("db connection");
	let collection = db.collection::<Document>(COLLECTION_NAME);

	let mut documents: Vec<AggregateCount> = vec![];
	let mut id: Document = doc!{};
	for (_index, column) in columns.iter().enumerate() {
		let mut dollared = String::from("$"); dollared.push_str(column);
		id.insert(column.to_owned(), dollared);
	}
	let mut cursor = collection.aggregate([
		doc!{
			"$group": doc!{
				"_id": id,
				"quantity": doc!{
					"$sum": 1 as i32,
				},
			},
		},
		doc!{
			"$sort": doc!{
				"quantity": -1 as i32,
			},
		},
	], None).await.expect("count by column cursor");
	while cursor.advance().await.unwrap() {
		let row: Document = cursor.deserialize_current().expect("count by column data");
		let mut columns: Vec<(String, String)> = vec![];
		let id = row.get_document("_id").expect("count by column id");
		for (_index, (key, value)) in id.iter().enumerate() {
			let column_key = key.clone();
			let column_value = value.as_str().expect("count by column id value as str").to_owned();
			columns.push((column_key, column_value));
		}
		let quantity: u32 = row.get_i32("quantity").expect("count by column quantity").try_into().expect("count by column quantity as u32");
		documents.push(AggregateCount { columns, quantity });
	}

	Ok(documents)
}

#[tokio::main]
async fn main() {
	let menu = Menu::new()
		.add_item(CustomMenuItem::new("show_page_data", "Data"))
		.add_item(CustomMenuItem::new("show_page_statistics", "Statistics"))
	;
	fn handle_menu_event(event: WindowMenuEvent) {
		match event.menu_item_id() {
			"show_page_data" => event.window().emit("change_page", ChangePagePayload { page: "data".into() }).expect("emit change page event"),
			"show_page_statistics" => event.window().emit("change_page", ChangePagePayload { page: "statistics".into() }).expect("emit change page event"),
			_ => {},
		}
	}

	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![
			db_connect,
			get_documents,
			save_document,
			remove_document,
			star_document,
			get_count_by_columns,
		])
		.menu(menu).on_menu_event(handle_menu_event)
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
