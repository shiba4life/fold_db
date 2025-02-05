mod types;
mod schema_mapper;
mod dsl;

#[cfg(test)]
mod tests;

pub use types::MappingRule;
pub use schema_mapper::SchemaMapper;
pub use dsl::parse_mapping_dsl;
