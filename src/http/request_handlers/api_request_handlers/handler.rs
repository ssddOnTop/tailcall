use serde::de::Error;
use serde_json::{Map, Value};
use crate::blueprint::{Definition, FieldDefinition, is_scalar};

pub struct ApiReqHandler{
    map: Value
}

impl ApiReqHandler {
    pub fn init(definitions: &Vec<Definition>) -> Result<Self, serde_json::Error> {
        let map = build_definition_map(definitions)?;
        Ok(Self{ map })
    }

}

fn build_definition_map(definitions: &Vec<Definition>) -> Result<Value, serde_json::Error> {
    // nothing fancy here, it just iterates over all the nested values and converts it to a map.
    let mut definition_map = Map::new();
    let mut current_definition = &mut definition_map;

    for definition in definitions {
        current_definition = current_definition
            .entry(definition.name())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .unwrap();

        for field in get_fields(definition).unwrap_or(&vec![]) {
            if is_scalar(field.of_type.name()) {
                current_definition.insert(field.name.clone(), Value::Null);
            } else {
                current_definition.insert(field.name.clone(), Value::String(field.of_type.name().to_string()));
            }
        }
        current_definition = &mut definition_map;
    }

    create_query_map(&definition_map, definition_map.get("Query").unwrap().clone())
}

pub fn get_fields(definition: &Definition) -> Option<&Vec<FieldDefinition>> {
    match definition {
        Definition::ObjectTypeDefinition(f) => Some(&f.fields),
        _ => None,
    }
}

fn create_query_map(definition_map: &Map<String, Value>, query_value: Value) -> Result<Value, serde_json::Error> {
    let mut result_map = Map::new();

    for (key, val) in query_value.as_object().ok_or(serde_json::Error::custom(format!(
        "The field: {} is not an json",
        query_value
    )))? {
        match val {
            Value::String(subtype) => {
                let mapped_value = create_query_map(definition_map, definition_map.get(subtype).unwrap().clone());
                result_map.insert(key.clone(), mapped_value?);
            }
            _ => {
                result_map.insert(key.clone(), Value::Null);
            }
        }
    }

    Ok(Value::Object(result_map))
}
