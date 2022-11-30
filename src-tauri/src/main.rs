#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

use tauri::{CustomMenuItem, Menu, WindowMenuEvent};
use mongodb::{bson::{doc, Document}, options::{ClientOptions, FindOptions, Collation}, Client, Database, Collection};
use std::{time::{SystemTime, UNIX_EPOCH}, sync::Mutex};
use anyhow::{anyhow, Error, Result};
use bson::{oid::ObjectId, spec::ElementType, Regex};
use std::collections::HashMap;
use maplit::hashmap;

static COLLECTION_NAME: Mutex<Option<String>> = Mutex::new(None);
static DATABASE: Mutex<Option<Database>> = Mutex::new(None);

#[derive(Clone, serde::Serialize)]
struct ChangePagePayload {
	page: String,
}

#[tauri::command]
async fn db_connect(connection_string: &str, database_name: &str, collection_name: &str) -> tauri::Result<bool> {
	let attempt = connect_client(connection_string).await;
	match attempt {
		Ok(client) => {
			let mut set_database = DATABASE.lock().unwrap();
			*set_database = Some(client.database(database_name));
			let mut set_collection_name = COLLECTION_NAME.lock().unwrap();
			*set_collection_name = Some(String::from(collection_name));
			return Ok(true);
		},
		Err(_msg) => {
			return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Could not connect to Database."))));
		},
	}
}

async fn connect_client(connection_string: &str) -> Result<Client, Error> {
	//! all that time just to fucking fail. i hate this db
	////let credential = match username.is_some() {
	////	true => Some(Credential::builder().username(username).password(password).source(Some(String::from("admin"))).build()),
	////	false => None,
	////};
	////let client_options = ClientOptions::builder()
	////	.hosts(vec![ServerAddress::Tcp { host: String::from(host), port: port }])
	////	.app_name(Some(String::from("MongoDB Compass")))
	////	.direct_connection(Some(true))
	////	.retry_writes(Some(true))
	////	.read_concern(Some(ReadConcern::majority()))
	////	.selection_criteria(Some(mongodb::options::SelectionCriteria::ReadPreference(mongodb::options::ReadPreference::Primary)))
	////	.write_concern(Some(WriteConcern::builder().w(Some(mongodb::options::Acknowledgment::Majority)).build()))
	////	.repl_set_name(replica_set_name)
	////	.credential(credential)
	////	.connect_timeout(Some(core::time::Duration::new(5, 0)))
	////	.server_selection_timeout(Some(core::time::Duration::new(5, 0)))
	////	.tls(Some(Tls::Enabled(TlsOptions::builder().allow_invalid_certificates(Some(true)).build())))
	////	.build()
	////;

	let client = Client::with_options(ClientOptions::parse(connection_string).await.expect("parsed connection string")).expect("database client");

	// Attempt connection
	match client.database("admin").run_command(doc!{"ping": 1}, None).await {
		Ok(_) => Ok(client),
		Err(error) => {
			eprintln!("{:#?}", error);
			Err(anyhow!("Could not connect to database."))
		}
	}
}

fn get_collection() -> Result<Collection<Document>, Error> {
	if COLLECTION_NAME.lock().unwrap().as_ref().is_none() {
		return Err(anyhow!("No collection name specified."))
	}
	match DATABASE.lock().unwrap().as_ref() {
		Some(database) => Ok(database.collection::<Document>(COLLECTION_NAME.lock().unwrap().as_ref().unwrap())),
		None => Err(anyhow!("Could not get database connection.")),
	}
}

#[tauri::command]
async fn save_document(
	id: Option<&str>, tsu: Option<&str>, tsa: Option<&str>,
	the_type: Option<&str>, tags: Option<Vec<&str>>, volume: Option<&str>, page: Option<&str>,
	cause: Option<&str>, effects: Option<Vec<&str>>, starred: Option<bool>,
) -> tauri::Result<()> {
	// Generate timestamp
	let start = SystemTime::now();
	let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("duration since the epoch");

	let collection: Collection<Document>;
	let get_collection = get_collection();
	match get_collection {
		Ok(c) => { collection = c },
		Err(error) => { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from(error.to_string())))); }
	}
	
	let mut doc: Document;
	let mut undoc: Document = doc!{};

	// Fill created or updated depending on operation (insert/update)
	match id {
		Some(_) => {
			doc = doc!{ "updated": since_the_epoch.as_secs() as u32 };
		},
		None => {
			doc = doc!{ "created": since_the_epoch.as_secs() as u32 };
		},
	};

	let fields = hashmap!{
		"tsu" => &tsu,
		"tsa" => &tsa,
		"type" => &the_type,
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
				// If field value is empty and this is an update operation, unset
				if id != None {
					undoc.insert(field_name, "");
				};
			},
		};
	}

	let array_fields = hashmap!{
		"tags" => &tags,
		"effects" => &effects,
	};
	for (&field_name, variable_ref) in &array_fields {
		match variable_ref {
			Some(the_array) => {
				if the_array.len() > 0 {
					doc.insert(field_name, the_array);
				} else {
					if id != None { undoc.insert(field_name, ""); }
				}
			},
			None => {
				// If field value is empty and this is an update operation, unset
				if id != None {
					undoc.insert(field_name, "");
				};
			},
		};
	}

	if starred.is_some() {
		doc.insert("starred", starred.unwrap());
	}
	
	if id == None {
		// Insert operation
		let result = collection.insert_one(doc, None).await;
		if result.is_err() { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Insert error.")))); }
	} else {
		// Update operation
		// Create container document
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
	let collection: Collection<Document>;
	let get_collection = get_collection();
	match get_collection {
		Ok(c) => { collection = c },
		Err(error) => { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from(error.to_string())))); }
	}

	let oid: ObjectId = ObjectId::parse_str(id.unwrap()).unwrap();
	let result = collection.delete_one(doc!{ "_id": oid }, None).await;
	if result.is_err() { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Delete error.")))); }
	Ok(())
}

#[tauri::command]
async fn get_documents(page: Option<u64>, size: Option<u64>, filters: Option<HashMap<String, String>>) -> tauri::Result<(Vec<Document>, u64, u64)> {
	let collection: Collection<Document>;
	let get_collection = get_collection();
	match get_collection {
		Ok(c) => { collection = c },
		Err(error) => { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from(error.to_string())))); }
	}

	// Construct search criteria document
	let mut search: Option<Document> = None;
	if !filters.is_none() {
		let mut filter_docs: Vec<Document> = vec![];
		for (key, value) in filters.unwrap().into_iter() {
			if value.len() < 1 { continue; }

			match key.as_str() {
				"volume" | "type" => {
					// Match exactly
					filter_docs.push(doc!{ key: value });
				},
				"page" => {
					// Get if starting matches
					filter_docs.push(doc!{ key: bson::Regex{ pattern: "^".to_owned() + regex::escape(&value).as_str(), options: String::from("i") } });
				},
				"tags" => {
					let mut all_tags: Vec<Regex> = vec![];
					let mut in_tags: Vec<Regex> = vec![];
					let mut nin_tags: Vec<Regex> = vec![];
					// Split by space
					let split_tags = &value.split_whitespace().collect::<Vec<&str>>();
					for split_value in split_tags {
						// Get if contains OR doesn't contain
						match split_value.chars().nth(0).unwrap() {
							'?' => { // Question mark: in
								let mut new_split_value = String::from(split_value.to_owned());
								new_split_value.remove(0);
								if new_split_value.len() > 0 {
									in_tags.push(Regex{
										pattern: regex::escape(&new_split_value),
										options: String::from("i")
									});
								}
							},
							'-' => { // Minus: nin
								let mut new_split_value = String::from(split_value.to_owned());
								new_split_value.remove(0);
								if new_split_value.len() > 0 {
									nin_tags.push(Regex{
										pattern: regex::escape(&new_split_value),
										options: String::from("i")
									});
								}
							},
							_ => { // Everything else: all
								let owned_split_value = String::from(split_value.to_owned());
								all_tags.push(Regex{
									pattern: regex::escape(&owned_split_value),
									options: String::from("i")
								});
							},
						}
					}
					let mut filter_doc = doc!{};
					if all_tags.len() > 0 {
						filter_doc.insert("$all", all_tags);
					}
					if in_tags.len() > 0 {
						filter_doc.insert("$in", in_tags);
					}
					if nin_tags.len() > 0 {
						filter_doc.insert("$nin", nin_tags);
					}
					filter_docs.push(doc!{ key: filter_doc });
				},
				"text" => {
					// Get if contains in tsa or tsu
					filter_docs.push(doc!{ "$or": [doc!{ "tsa": Regex{ pattern: regex::escape(&value), options: String::from("im") } }, doc!{ "tsu": Regex{ pattern: regex::escape(&value), options: String::from("im") } }] });
				},
				"cause" | "effects" | "starred" => {
					// Get if has or has no cause/effects
					let the_bool = match &value[..] {
						"y" => Some(true),
						"n" => Some(false),
						_ => None,
					};
					if the_bool.is_none() { continue; }
					if key == "starred" {
						match the_bool.unwrap() {
							true => filter_docs.push(doc!{ &key: true }),
							false => filter_docs.push(doc!{ "$or": [doc!{ &key: false }, doc!{ &key: { "$exists": false } }] }),
						}
					} else {
						filter_docs.push(doc!{ &key: { "$exists": the_bool.unwrap() } });
					}
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

	Ok((documents, page, total_filtered_count))
}

#[tauri::command]
async fn star_document(id: &str) -> tauri::Result<bool> {
	let collection: Collection<Document>;
	let get_collection = get_collection();
	match get_collection {
		Ok(c) => { collection = c },
		Err(error) => { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from(error.to_string())))); }
	}

	let oid: ObjectId = ObjectId::parse_str(id).expect("valid oid");
	let search = collection.find_one(doc!{ "_id": oid }, None).await.unwrap();

	if search.is_none() {
		return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Document not found."))));
	}

	let doc: Document = search.unwrap();
	let starred: bool = doc.get_bool("starred").unwrap_or(false);

	let setdoc = if !starred { doc!{ "$set": doc!{ "starred": true } } } else { doc!{ "$unset": doc!{ "starred": false } } };
	let result = collection.find_one_and_update(doc!{ "_id": oid, }, setdoc, None).await;
	if result.is_err() {
		eprintln!("{:#?}", result.err().unwrap());
		return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from("Update error."))));
	}
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
	let collection: Collection<Document>;
	let get_collection = get_collection();
	match get_collection {
		Ok(c) => { collection = c },
		Err(error) => { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from(error.to_string())))); }
	}

	let mut documents: Vec<AggregateCount> = vec![];
	let mut unwinds: Vec<String> = vec![]; // For keeping track of tags that will be deconstructed
	let mut id: Document = doc!{};
	for (_index, column) in columns.iter().enumerate() {
		if column.len() < 1 { continue; } // Cancel if empty string

		let mut process_column = String::from(column.to_owned());
		// If starts with "unwind:", mark column as unwind and remove that "unwind:" part from the string
		if process_column.starts_with("unwind:") {
			process_column = process_column.chars().skip(7).take(process_column.len() - 7).collect();
			unwinds.push(process_column.clone());
		}
		let mut dollared_column_name = String::from("$"); dollared_column_name.push_str(&process_column);
		id.insert(&process_column, dollared_column_name);
	}

	let mut pipeline: Vec<Document> = vec![];
	// Tell to unwind columns if specified
	for unwind_column in unwinds {
		pipeline.push(doc!{
			"$unwind": doc!{
				"path": String::from("$") + &unwind_column,
				"preserveNullAndEmptyArrays": false,
			},
		});
	}
	// Group by columns
	pipeline.push(doc!{
		"$group": doc!{
			"_id": id,
			"quantity": doc!{
				"$sum": 1 as i32,
			},
		},
	});
	// Sort by quantity in descending order
	pipeline.push(doc!{
		"$sort": doc!{
			"quantity": -1 as i32,
		},
	});

	let mut cursor = collection.aggregate(pipeline, None).await.expect("count by column cursor");
	while cursor.advance().await.unwrap() {
		let row: Document = cursor.deserialize_current().expect("count by column data");
		let mut columns: Vec<(String, String)> = vec![]; // Tuple of column names (identifier, human_readable)
		let aggregate_id = row.get_document("_id").expect("count by column id");
		for (_index, (key, value)) in aggregate_id.iter().enumerate() {
			let column_key = key.clone();
			if value.element_type() == ElementType::Array {
				// If aggregate ID is an array, mix them together into one column
				let column_vec = value.as_array().expect("count by column id value as array").to_owned();
				let mut value_string: Vec<String> = vec![];
				for column_value in column_vec {
					let column_string = column_value.as_str().expect("count by column id value array as string").to_owned();
					value_string.push(column_string);
				}
				columns.push((column_key, value_string.join(" ")));
			} else {
				let column_value = value.as_str().expect("count by column id value as str").to_owned();
				columns.push((column_key, column_value));
			}
		}
		let quantity: u32 = row.get_i32("quantity").expect("count by column quantity").try_into().expect("count by column quantity as u32");
		documents.push(AggregateCount { columns, quantity });
	}

	Ok(documents)
}

#[tauri::command]
async fn get_documents_by_tags(tags: Vec<&str>) -> tauri::Result<HashMap<String, Vec<Document>>> {
	let collection: Collection<Document>;
	let get_collection = get_collection();
	match get_collection {
		Ok(c) => { collection = c },
		Err(error) => { return Err(tauri::Error::FailedToExecuteApi(tauri::api::Error::Command(String::from(error.to_string())))); }
	}

	// HashMap to store return value
	let mut final_map: HashMap<String, Vec<Document>> = hashmap!{};
	// FindOptions
	let collation = Collation::builder().locale("en_US").numeric_ordering(true).build();
	let sort = doc!{ "volume": 1, "page": 1, "created": 1 };
	let options = FindOptions::builder().sort(sort).collation(collation).build();

	// Get for each tags
	for tag in tags.iter() {
		let mut documents: Vec<Document> = vec!{};
		
		let mut cursor = collection.find(doc!{ "tags": tag }, options.clone()).await.unwrap();
		while cursor.advance().await.unwrap() {
			documents.push(cursor.deserialize_current().unwrap());
		}

		// Append to HashMap
		final_map.insert(String::from(*tag), documents);
	}

	return Ok(final_map);
}

#[tokio::main]
async fn main() {
	let menu = Menu::new()
		.add_item(CustomMenuItem::new("show_page_data", "Data"))
		.add_item(CustomMenuItem::new("show_page_statistics", "Statistics"))
		.add_item(CustomMenuItem::new("show_page_export", "Export"))
	;
	fn handle_menu_event(event: WindowMenuEvent) {
		match event.menu_item_id() {
			"show_page_data" => event.window().emit("change_page", ChangePagePayload { page: "data".into() }).expect("emit change page event"),
			"show_page_statistics" => event.window().emit("change_page", ChangePagePayload { page: "statistics".into() }).expect("emit change page event"),
			"show_page_export" => event.window().emit("change_page", ChangePagePayload { page: "export".into() }).expect("emit change page event"),
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
			get_documents_by_tags,
		])
		.menu(menu).on_menu_event(handle_menu_event)
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
