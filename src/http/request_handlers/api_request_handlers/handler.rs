use serde::de::Error;
use serde_json::{Map, Value};
use crate::blueprint::{Definition, FieldDefinition, InputFieldDefinition, is_scalar};

pub struct ApiReqHandler{
    map: Value
}

impl ApiReqHandler {
    pub fn init(definitions: &Vec<Definition>) -> Result<Self, serde_json::Error> {
        let map = build_definition_map(definitions)?;
        println!("{map:#}");
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
            let mut arr = grt_args(&field.args);
            if !is_scalar(field.of_type.name()) {
                arr.as_array_mut().unwrap_or(&mut vec![]).push(Value::String(field.of_type.name().to_string()));
            }
            current_definition.insert(field.name.clone(), arr);

        }
        current_definition = &mut definition_map;
    }
    println!("{:#?}",definition_map);
    create_query_map(&definition_map, definition_map.get("Query").unwrap().clone())
}

fn grt_args(info_definitions: &Vec<InputFieldDefinition>) -> Value {
    let mut v = vec![];
    if info_definitions.is_empty() {
        return Value::Array(v);
    }
    for i in info_definitions {
        let mut m = Map::new();
        m.insert("name".to_string(),Value::String(i.name.clone()));
        m.insert("type".to_string(),Value::String(i.of_type.name().to_string()));
        m.insert("nullable".to_string(),Value::Bool(i.of_type.is_nullable()));
        v.push(Value::Object(m));
    }
    Value::Array(v)
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
            Value::Array(subtype) => {
                if subtype.last().unwrap_or(&Value::Null).is_string() {
                    let mapped_value = create_query_map(definition_map, definition_map.get(subtype.last().unwrap().as_str().unwrap()).unwrap().clone());
                    result_map.insert(key.clone(), mapped_value?);
                }else {
                    result_map.insert(key.clone(), Value::Null);
                }
            }
            _ => {
                result_map.insert(key.clone(), Value::Null);
            }
        }
    }
    Ok(Value::Object(result_map))
}
