use std::cell::RefCell;

use serde::{Deserialize, Serialize};

thread_local! {
    static WITH_COMMENTS: RefCell<bool> = RefCell::new(false);
}

pub fn json_to_schema(json: &str, with_comments: bool) -> String {
    WITH_COMMENTS.with(|f| *f.borrow_mut() = with_comments);

    let schema: Container = serde_json::from_str(json).unwrap();
    let mut schema_str = String::new();

    write_schema(&mut schema_str, &schema.data.__schema);

    schema_str
}

fn write_schema(writer: &mut String, schema: &Schema) {
    write_derectives(writer, &schema.derectives);
    write_types(writer, &schema.types);
}

fn write_derectives(writer: &mut String, derectives: &Vec<Derective>) {
    for derective in derectives {
        write_derective(writer, derective);
    }
}

fn write_derective(writer: &mut String, derective: &Derective) {
    if let Some(description) = &derective.description {
        writer.push_str("description: ");
        writer.push_str(description);
        writer.push_str("\n");
    }
    writer.push_str("directive @");
    writer.push_str(&derective.name);
    writer.push_str("(");
    for (i, arg) in derective.args.iter().enumerate() {
        write_input_value(writer, arg);
        if i < derective.args.len() - 1 {
            writer.push_str(", ");
        }
    }
    writer.push_str(") on ");
    for (i, location) in derective.locations.iter().enumerate() {
        writer.push_str(location);
        if i < derective.locations.len() - 1 {
            writer.push_str(" | ");
        }
    }
    writer.push_str("\n\n");
}

fn write_input_value(writer: &mut String, input_value: &InputValue) {
    writer.push_str(&input_value.name);
    writer.push_str(": ");
    write_type_ref(writer, &input_value.r#type);
    if let Some(default_value) = &input_value.default_value {
        writer.push_str(" = ");
        writer.push_str(default_value);
    }
}

fn write_types(writer: &mut String, types: &Vec<Type>) {
    for type_ in types {
        write_type(writer, type_);
    }
}

fn write_type(writer: &mut String, type_: &Type) {
    if let Some(description) = &type_.description {
        write_comment(writer, description);
    }
    match type_.kind {
        TypeKind::Scalar => {
            writer.push_str("scalar ");
            writer.push_str(&type_.name);
            writer.push_str("\n");
        }
        TypeKind::Object => {
            writer.push_str("type ");
            writer.push_str(&type_.name);
            writer.push_str(" {\n");
            if let Some(fields) = &type_.fields {
                for field in fields {
                    write_field(writer, field);
                }
            }
            writer.push_str("}\n");
        }
        TypeKind::Interface => {
            writer.push_str("interface ");
            writer.push_str(&type_.name);
            writer.push_str(" {\n");
            if let Some(fields) = &type_.fields {
                for field in fields {
                    write_field(writer, field);
                }
            }
            writer.push_str("}\n");
        }
        TypeKind::Union => {
            writer.push_str("union ");
            writer.push_str(&type_.name);
            writer.push_str(" = ");
            if let Some(possible_types) = &type_.possible_types {
                for (i, possible_type) in possible_types.iter().enumerate() {
                    writer.push_str(&possible_type.name);
                    if i < possible_types.len() - 1 {
                        writer.push_str(" | ");
                    }
                }
            }
            writer.push_str("\n");
        }
        TypeKind::Enum => {
            writer.push_str("enum ");
            writer.push_str(&type_.name);
            writer.push_str(" {\n");
            if let Some(enum_values) = &type_.enum_values {
                for enum_value in enum_values {
                    writer.push_str("  ");
                    writer.push_str(&enum_value.name);
                    writer.push_str("\n");
                }
            }
            writer.push_str("}\n");
        }
        TypeKind::InputObject => {
            writer.push_str("input ");
            writer.push_str(&type_.name);
            writer.push_str(" {\n");
            if let Some(input_fields) = &type_.input_fields {
                for input_field in input_fields {
                    writer.push_str("  ");
                    write_input_field(writer, input_field);
                }
            }
            writer.push_str("}\n");
        }
        TypeKind::List => todo!(),
        TypeKind::NonNull => todo!(),
    }
}

fn write_field(writer: &mut String, field: &Field) {
    if let Some(description) = &field.description {
        writer.push_str("  ");
        write_comment(writer, description);
    }
    writer.push_str("  ");
    writer.push_str(&field.name);
    if field.args.as_ref().map(|args| args.len()).unwrap_or(0) > 0 {
        writer.push_str("(");
        for (i, arg) in field.args.as_ref().unwrap().iter().enumerate() {
            write_arg(writer, arg);
            if i < field.args.as_ref().unwrap().len() - 1 {
                writer.push_str(", ");
            }
        }
        writer.push_str(")");
    }
    writer.push_str(": ");
    write_type_ref(writer, &field.r#type);
    writer.push_str("\n");
}

fn write_arg(writer: &mut String, arg: &Arg) {
    writer.push_str(&arg.name);
    writer.push_str(": ");
    write_type_ref(writer, &arg.r#type);
    if let Some(default_value) = &arg.default_value {
        writer.push_str(" = ");
        writer.push_str(default_value);
    }
}

fn write_type_ref(writer: &mut String, type_ref: &TypeRef) {
    match &type_ref.kind {
        TypeKind::Scalar => {
            writer.push_str(&type_ref.name.as_ref().unwrap());
        }
        TypeKind::Object => {
            writer.push_str(&type_ref.name.as_ref().unwrap());
        }
        TypeKind::Interface => {
            writer.push_str(&type_ref.name.as_ref().unwrap());
        }
        TypeKind::Union => {
            writer.push_str(&type_ref.name.as_ref().unwrap());
        }
        TypeKind::Enum => {
            writer.push_str(&type_ref.name.as_ref().unwrap());
        }
        TypeKind::InputObject => {
            writer.push_str(&type_ref.name.as_ref().unwrap());
        }
        TypeKind::List => {
            writer.push_str("[");
            write_type_ref(writer, &type_ref.of_type.as_ref().unwrap());
            writer.push_str("]");
        }
        TypeKind::NonNull => {
            write_type_ref(writer, &type_ref.of_type.as_ref().unwrap());
            writer.push_str("!");
        }
    }
}

fn write_input_field(writer: &mut String, input_field: &InputField) {
    writer.push_str(&input_field.name);
    writer.push_str(": ");
    write_type_ref(writer, &input_field.r#type);
    if let Some(default_value) = &input_field.default_value {
        writer.push_str(" = ");
        writer.push_str(default_value);
    }
    writer.push_str("\n");
}

fn write_comment(writer: &mut String, comment: &str) {
    if !WITH_COMMENTS.with(|f| *f.borrow()) {
        return;
    }

    writer.push_str(r#"""""#);
    writer.push_str(comment);
    if comment.ends_with("\"") {
        writer.push_str(" ");
    }
    writer.push_str(r#"""""#);
    writer.push_str("\n");
}

#[derive(Debug, Serialize, Deserialize)]
struct Container {
    data: ContainerData,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContainerData {
    __schema: Schema,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Schema {
    query_type: Option<QueryType>,
    mutation_type: Option<MutationType>,
    subscription_type: Option<SubscriptionType>,
    types: Vec<Type>,
    #[serde(default)]
    derectives: Vec<Derective>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryType {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MutationType {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubscriptionType {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Type {
    kind: TypeKind,
    name: String,
    description: Option<String>,
    fields: Option<Vec<Field>>,
    input_fields: Option<Vec<InputField>>,
    interfaces: Option<Vec<Interface>>,
    enum_values: Option<Vec<EnumValue>>,
    possible_types: Option<Vec<PossibleType>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum TypeKind {
    Scalar,
    Object,
    Interface,
    Union,
    Enum,
    InputObject,
    List,
    NonNull,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Field {
    name: String,
    description: Option<String>,
    args: Option<Vec<Arg>>,
    r#type: TypeRef,
    is_deprecated: bool,
    deprecation_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InputField {
    name: String,
    description: Option<String>,
    r#type: TypeRef,
    default_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Interface {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnumValue {
    name: String,
    description: Option<String>,
    is_deprecated: bool,
    deprecation_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PossibleType {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Arg {
    name: String,
    description: Option<String>,
    r#type: TypeRef,
    default_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TypeRef {
    kind: TypeKind,
    name: Option<String>,
    of_type: Option<Box<TypeRef>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Derective {
    name: String,
    description: Option<String>,
    locations: Vec<String>,
    args: Vec<InputValue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InputValue {
    name: String,
    description: Option<String>,
    r#type: TypeRef,
    default_value: Option<String>,
}
