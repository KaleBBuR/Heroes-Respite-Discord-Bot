use std::env;

use serenity::prelude::TypeMapKey;
use mongodb::{Collection, Client};
use mongodb::bson::{doc, Document};
use serde::{Serialize, Deserialize};
use serenity::prelude::Context;
use mongodb::options::FindOneAndReplaceOptions;
use crate::party_groups::Group;

pub struct Database;

#[derive(Debug, Serialize, Deserialize)]
// You need to return a Struct when getting information from the MongoDB Database, and this will be
// the struct that will implement all the fields for the bot.
pub(crate) struct DatabaseServer {
    // Fields
    _id: i64,
    owner_id: i64,
    current_groups: Vec<Group>,
    party_owners: Vec<i64>
}

impl TypeMapKey for Database {
    type Value = Client;
}

impl DatabaseServer {
    pub(crate) async fn get_or_insert_new(
        ctx: &Context,
        _id: i64,
        owner_id: Option<i64>
    ) -> DatabaseServer {
        let get_result = DatabaseServer::get(ctx, _id).await;

        if get_result == None && owner_id.is_some() {
            bson::from_document(
                DatabaseServer::insert_or_replace(ctx, DatabaseServer {
                    _id,
                    owner_id: owner_id.unwrap(),
                    current_groups: Vec::new(),
                    party_owners: Vec::new()
                }).await
            ).unwrap()
        } else {
            bson::from_document(get_result.unwrap()).unwrap()
        }
    }

    pub(crate) async fn get(ctx: &Context, _id: i64) -> Option<Document> {
        let document_id = doc! { "_id": _id };
        let document = DatabaseServer::get_collection(ctx)
            .await
            .find_one(document_id, None)
            .await
            .unwrap();

        document
    }

    pub(crate) async fn insert_or_replace(
        ctx: &Context,
        database_guild: DatabaseServer
    ) -> Document {
        let new_document = bson::to_document(&database_guild).unwrap();

        let mut replace_options = FindOneAndReplaceOptions::default();
        replace_options.upsert = Some(true);

        let collection = DatabaseServer::get_collection(ctx).await;
        // Find and replace the document and return it
        match collection.find_one_and_replace(
            doc! { "_id": database_guild._id },
            new_document,
            replace_options
        ).await.unwrap() {
            Some(document) => document,
            None => {
                collection.find_one(
                    doc! { "_id": database_guild._id },
                    None
                )
                .await
                .unwrap()
                .unwrap()
            }
        }
    }

    pub(crate) async fn delete(
        ctx: &Context,
        id: i64
    ) -> mongodb::error::Result<Option<Document>> {
        let document_id = doc! { "_id": id };
        DatabaseServer::get_collection(ctx).await.find_one_and_delete(document_id, None).await
    }

    pub(crate) async fn get_collection(ctx: &Context) -> Collection {
        let mongo_database = env::var("MONGO_DB").unwrap();
        let database = ctx.data
            .read()
            .await
            .get::<Database>()
            .unwrap()
            .database(&mongo_database);

        database.collection("Servers")
    }

    pub(crate) async fn party_owner(ctx: &Context, _id: i64, party_owner_id: i64) -> bool {
        let dbs = DatabaseServer::get_or_insert_new(ctx, _id, None).await;
        for curr_party_owner in dbs.party_owners {
            if curr_party_owner == party_owner_id {
                return true
            }
        }

        false
    }

    pub(crate) async fn add_party(&mut self, party_group: Group) {
        self.current_groups.push(party_group);
    }

    pub(crate) async fn edit_party(&mut self, party_group: Group) {
        for (i, curr_group) in self.current_groups.iter().enumerate() {
            if curr_group.party_owner == party_group.party_owner {
                self.current_groups.remove(i);
                self.current_groups.push(party_group);
                break
            }
        }
    }
}