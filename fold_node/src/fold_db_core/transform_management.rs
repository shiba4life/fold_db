use super::FoldDB;
use crate::schema::types::{Transform, TransformRegistration};
use crate::schema::{Schema, SchemaError};
use crate::schema::types::field::common::Field;
use log::{error, info};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

/// Result type for parsing transform inputs.
type TransformInputResult = Result<(Vec<(String, String)>, Vec<String>), SchemaError>;

/// Parameters used when registering a transform.
pub(super) struct TransformRegistrationParams {
    input_arefs: Vec<String>,
    input_names: Vec<String>,
    trigger_fields: Vec<String>,
    output_aref: String,
}

impl FoldDB {
    pub fn register_transform(&mut self, registration: TransformRegistration) -> Result<(), SchemaError> {
        self.transform_manager.register_transform(registration)
    }

    pub(super) fn parse_output_field(
        &self,
        schema: &Schema,
        field_name: &str,
        field: &crate::schema::types::FieldVariant,
        transform: &Transform,
    ) -> Result<String, SchemaError> {
        let (out_schema_name, out_field_name) = match transform.get_output().split_once('.') {
            Some((s, f)) => (s.to_string(), f.to_string()),
            None => (schema.name.clone(), field_name.to_string()),
        };

        info!("Parsing output field for transform with output: {} -> {}.{}",
              transform.get_output(), out_schema_name, out_field_name);

        if out_schema_name == schema.name && out_field_name == field_name {
            info!("Output field matches current field: {}.{}", schema.name, field_name);
            let aref = field.ref_atom_uuid().ok_or_else(|| {
                error!("Field {}.{} missing atom reference", schema.name, field_name);
                SchemaError::InvalidData(format!("Field {} missing atom reference", field_name))
            })?.clone();
            info!("Found ARef for {}.{}: {}", schema.name, field_name, aref);
            Ok(aref)
        } else {
            info!("Output field is different schema/field: {}.{}", out_schema_name, out_field_name);
            match self
                .schema_manager
                .get_schema(&out_schema_name)?
                .and_then(|s| s.fields.get(&out_field_name).cloned())
            {
                Some(of) => {
                    let aref = of.ref_atom_uuid().ok_or_else(|| {
                        error!("Field {}.{} missing atom reference", out_schema_name, out_field_name);
                        SchemaError::InvalidData(format!(
                            "Field {}.{} missing atom reference",
                            out_schema_name, out_field_name
                        ))
                    })?.clone();
                    info!("Found ARef for {}.{}: {}", out_schema_name, out_field_name, aref);
                    Ok(aref)
                },
                None => {
                    info!("Output field {}.{} not found, using current field", out_schema_name, out_field_name);
                    let aref = field.ref_atom_uuid().ok_or_else(|| {
                        error!("Field {} missing atom reference", field_name);
                        SchemaError::InvalidData(format!("Field {} missing atom reference", field_name))
                    })?.clone();
                    info!("Using ARef from current field {}: {}", field_name, aref);
                    Ok(aref)
                },
            }
        }
    }

    pub(super) fn collect_input_arefs(
        &self,
        schema: &Schema,
        transform: &Transform,
        cross_re: &Regex,
    ) -> TransformInputResult {
        let mut input_pairs = Vec::new();
        let mut input_arefs = Vec::new();
        let mut trigger_fields = Vec::new();
        let mut seen_cross = std::collections::HashSet::new();

        let inputs = transform.get_inputs();
        if !inputs.is_empty() {
            for input in inputs {
                if let Some((schema_name, field_dep)) = input.split_once('.') {
                    seen_cross.insert(field_dep.to_string());
                    trigger_fields.push(format!("{}.{}", schema_name, field_dep));
                    if let Some(dep_schema) = self.schema_manager.get_schema(schema_name)? {
                        if let Some(dep_field) = dep_schema.fields.get(field_dep) {
                            if let Some(dep_aref) = dep_field.ref_atom_uuid() {
                                input_arefs.push(dep_aref.clone());
                                input_pairs.push((dep_aref.clone(), format!("{}.{}", schema_name, field_dep)));
                            }
                        }
                    }
                }
            }
        } else {
            for cap in cross_re.captures_iter(&transform.logic) {
                let schema_name = cap[1].to_string();
                let field_dep = cap[2].to_string();
                seen_cross.insert(field_dep.clone());
                trigger_fields.push(format!("{}.{}", schema_name, field_dep));
                if let Some(dep_schema) = self.schema_manager.get_schema(&schema_name)? {
                    if let Some(dep_field) = dep_schema.fields.get(&field_dep) {
                        if let Some(dep_aref) = dep_field.ref_atom_uuid() {
                            input_arefs.push(dep_aref.clone());
                            input_pairs.push((dep_aref.clone(), format!("{}.{}", schema_name, field_dep)));
                        }
                    }
                }
            }
        }

        for dep in transform.analyze_dependencies() {
            if seen_cross.contains(&dep) {
                continue;
            }
            let schema_name = schema.name.clone();
            let field_dep = dep;

            trigger_fields.push(format!("{}.{}", schema_name, field_dep));

            if let Some(dep_schema) = self.schema_manager.get_schema(&schema_name)? {
                if let Some(dep_field) = dep_schema.fields.get(&field_dep) {
                    if let Some(dep_aref) = dep_field.ref_atom_uuid() {
                        input_arefs.push(dep_aref.clone());
                        input_pairs.push((dep_aref.clone(), format!("{}.{}", schema_name, field_dep)));
                    }
                }
            }
        }

        Ok((input_pairs, trigger_fields))
    }

    pub(super) fn register_transform_internal(
        &self,
        schema: &Schema,
        field_name: &str,
        transform: &Transform,
        params: TransformRegistrationParams,
    ) -> Result<(), SchemaError> {
        let transform_id = format!("{}.{}", schema.name, field_name);
        let mut trigger_fields = params.trigger_fields;
        trigger_fields.push(transform_id.clone());
        let registration = TransformRegistration {
            transform_id: transform_id.clone(),
            transform: transform.clone(),
            input_arefs: params.input_arefs,
            input_names: params.input_names,
            trigger_fields,
            output_aref: params.output_aref,
            schema_name: schema.name.clone(),
            field_name: field_name.to_string(),
        };
        self.transform_manager.register_transform(registration)?;
        let _ = self.transform_manager.execute_transform_now(&transform_id);
        Ok(())
    }

    pub(super) fn register_transforms_for_schema(&self, schema: &Schema) -> Result<(), SchemaError> {
        let cross_re = Regex::new(r"([A-Za-z0-9_]+)\.([A-Za-z0-9_]+)").unwrap();

        for (field_name, field) in &schema.fields {
            if let Some(transform) = field.transform() {
                let output_aref = self.parse_output_field(schema, field_name, field, transform)?;
                let (pairs, trigger_fields) =
                    self.collect_input_arefs(schema, transform, &cross_re)?;
                let mut input_arefs = Vec::new();
                let mut input_names = Vec::new();
                for (a, n) in pairs {
                    input_arefs.push(a);
                    input_names.push(n);
                }
                self.register_transform_internal(
                    schema,
                    field_name,
                    transform,
                    TransformRegistrationParams {
                        input_arefs,
                        input_names,
                        trigger_fields,
                        output_aref,
                    },
                )?;
            }
        }

        Ok(())
    }

    pub fn orchestrator_len(&self) -> Result<usize, SchemaError> {
        self.transform_orchestrator.len()
    }

    pub fn list_transforms(&self) -> Result<HashMap<String, Transform>, SchemaError> {
        self.transform_manager.list_transforms()
    }

    pub fn run_transform(&self, transform_id: &str) -> Result<Value, SchemaError> {
        self.transform_manager.execute_transform_now(transform_id)
    }
}
