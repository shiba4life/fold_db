use async_graphql::{EmptySubscription, Schema};
use std::sync::Arc;

use super::types::{MutationRoot, QueryRoot};
use crate::folddb::FoldDB;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema(fold_db: Arc<FoldDB>) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(fold_db)
        .finish()
}
