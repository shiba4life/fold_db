use actix_web::{web, App, HttpServer};
use std::sync::Arc;

use crate::store::Store;
use crate::schema::SchemaManager;
use super::handlers::{query_handler, write_handler};

pub struct ApiServer {
    schema_manager: Arc<SchemaManager>,
    store: Arc<Store>,
    bind_address: String,
}

impl ApiServer {
    pub fn new(schema_manager: Arc<SchemaManager>, store: Arc<Store>, bind_address: String) -> Self {
        ApiServer {
            schema_manager,
            store,
            bind_address,
        }
    }

    pub async fn run(&self) -> std::io::Result<()> {
        let schema_manager = self.schema_manager.clone();
        let store = self.store.clone();
        let bind_address = self.bind_address.clone();

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(schema_manager.clone()))
                .app_data(web::Data::new(store.clone()))
                .route("/api/query", web::post().to(query_handler))
                .route("/api/write", web::post().to(write_handler))
        })
        .bind(&bind_address)?
        .run()
        .await
    }
}
