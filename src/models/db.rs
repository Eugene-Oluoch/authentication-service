use mongodb::{Client, Collection, results::{InsertOneResult, DeleteResult, UpdateResult}, bson::{doc,Document}, error::Error};
use serde::Serialize;
use dotenv::dotenv;
use std::{env::var};


async fn create_collection<T>(client:&Client, collection:&str) -> Collection<T>{

  // TODO CREATE A DEFAULT DB_NAME IF ONE WASN'T SUPPLIED

  let db_name = var("MONGO_DB_NAME").expect("MONGO_DB_NAME must be set");
  client.database(db_name.as_str()).collection(collection)
}

pub async fn create_connection() -> Client {
  // LOADS ENVS
  dotenv().ok();
  let uri = var("MONGO_URI").expect("MONGO_URI must be set.");
  println!("{}",uri);
  Client::with_uri_str(uri).await.expect("Failed to initialize client.")
}


pub async fn insert_doc<T>(client:&Client, collection:&str,document:&T) -> Result<InsertOneResult, String>
where T: Serialize
{
  let col:Collection<T> = create_collection(client, collection).await;
  match col.insert_one(document,None).await{
    Ok(column) => Ok(column),
    Err(err) => Err(err.to_string())
  }

}


pub async fn get_one<T>(client:&Client,collection:&str,doc:Document) -> Result<Option<T>,Error>
where T:Serialize 
+ std::fmt::Debug 
+ for<'de> serde::Deserialize<'de> 
+ Unpin + Send + Sync
{
  let col:Collection<T> = create_collection(client, collection).await;
  col.find_one(doc,None).await
}


pub async fn update_many<T>(client:&Client,collection:&str,match_:Document,action:Document){
  let col:Collection<T> = create_collection(client,collection).await;
  println!("{:?}",col.update_many(match_,action,None).await.expect("testing"));
}

pub async fn update_one<T>(client:&Client, collection:&str,doc:Document,id:Document) -> Result<UpdateResult,Error>{
  let col:Collection<T> = create_collection(client,collection).await;
  col.update_one(id, doc, None).await
}


pub async fn delete_by_id<T>(client:&Client, collection:&str, id:&str) -> Result<DeleteResult,Error> {
  let col:Collection<T> = create_collection(client,collection).await;
  col.delete_one(doc! {"_id":id}, None).await
}