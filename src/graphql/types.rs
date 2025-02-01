use async_graphql::*;
use serde_json::Value;

#[derive(SimpleObject)]
pub struct FieldData {
    pub field: String,
    pub value: String,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    pub async fn field(&self, ctx: &Context<'_>, schema_name: String, field: String) -> Result<FieldData> {
        let fold_db = ctx.data::<std::sync::Arc<crate::folddb::FoldDB>>()?;
        let value = fold_db
            .get_field_value(&schema_name, &field)
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(FieldData {
            field,
            value: value.to_string(),
        })
    }

    pub async fn field_history(
        &self,
        ctx: &Context<'_>,
        schema_name: String,
        field: String,
    ) -> Result<Vec<String>> {
        let fold_db = ctx.data::<std::sync::Arc<crate::folddb::FoldDB>>()?;
        
        let internal_schema = fold_db
            .internal_schemas
            .get(&schema_name)
            .ok_or_else(|| Error::new("Schema not found"))?;
        let aref_uuid = internal_schema
            .fields
            .get(&field)
            .ok_or_else(|| Error::new("Field not found"))?;
        
        let history = fold_db
            .get_atom_history(aref_uuid)
            .map_err(|e| Error::new(e.to_string()))?;
        
        Ok(history.into_iter().map(|atom| atom.content).collect())
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    pub async fn update_field(
        &self,
        ctx: &Context<'_>,
        schema_name: String,
        field: String,
        value: String,
        source: String,
    ) -> Result<bool> {
        let fold_db = ctx.data::<std::sync::Arc<crate::folddb::FoldDB>>()?;
        
        let json_value: Value = serde_json::from_str(&value)
            .map_err(|e| Error::new(format!("Invalid JSON value: {}", e)))?;
        
        fold_db
            .update_field_value(&schema_name, &field, json_value, source)
            .map_err(|e| Error::new(e.to_string()))?;
        
        Ok(true)
    }
}
