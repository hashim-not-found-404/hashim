#![crate_type = "proc-macro"]
// extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree};
// use proc_macro2::TokenTree;
use quote::{ToTokens, quote};
use std::str::FromStr;
use syn::Ident;

const TABLE: &str = "table";
const FIELD: &str = "field";
const RECORD: &str = "record";

#[derive(Default, derive_more::Debug)]
struct SchemaAST {
    tables: Vec<Table>,
}

#[derive(PartialEq, Clone, Default, derive_more::Debug)]
struct Table {
    name: String,
    id: u16,
    fields: Vec<Field>,
}

#[derive(PartialEq, Clone, Default, derive_more::Debug)]
struct Field {
    name: String,
    id: u8,
    data_type: DataType,
    // is_record_id: bool,
}

#[derive(PartialEq, Clone, derive_more::Debug)]
enum DataType {
    SingleType(String),
    Record(Vec<String>),
}

impl Default for DataType {
    fn default() -> Self {
        Self::SingleType("".to_string())
    }
}

#[proc_macro]
pub fn make_schema(input: TokenStream) -> TokenStream {
    let schema = match generate_ast1(input) {
        Ok(o) => o,
        Err(e) => return e,
    };

    // dbg!(&schema);
    check_ast(&schema);
    make_graph(&schema);
    let code = generate_rust_code(&schema);

    // println!("{}", &code);
    code.into()
}

enum State {
    CheckIfTableOrFieldKeyWord,
    FieldKeyWord,
    CheckIfArrayOrEnumOrRecordKeyWordOrType,
    StoreTableId,
    StoreTableName,
    StoreFieldId,
    StoreFieldName,
    ProcessOpenRecord,
    StoreRecordType,
}

fn generate_ast1(input: TokenStream) -> Result<SchemaAST, TokenStream> {
    let mut schema = SchemaAST::default();
    let mut errors = quote! {};

    let mut state = State::CheckIfTableOrFieldKeyWord;
    let mut current_table = Table::default();
    let mut current_field = Field::default();

    for token in input.into_iter() {
        let mut make_error = |msg: String| {
            errors.extend(syn::Error::new(token.span().into(), msg).into_compile_error())
        };

        match state {
            State::ProcessOpenRecord => {
                if let TokenTree::Punct(_) = &token {
                    if &token.to_string() == "<" {
                        state = State::StoreRecordType;
                    } else {
                        make_error("expected angel prackets '<'".to_string());
                    }
                } else {
                    make_error("expected punctuation".to_string());
                }
            }
            State::StoreRecordType => {
                if let TokenTree::Ident(ident) = &token {
                    match current_field.data_type {
                        DataType::SingleType(_) => {}
                        DataType::Record(ref mut e) => e.push(ident.to_string()),
                    }
                    state = State::StoreRecordType;
                } else if let TokenTree::Punct(_) = &token {
                    if &token.to_string() == ">" {
                        state = State::CheckIfTableOrFieldKeyWord;
                    } else {
                        make_error("expected angel prackets '>'".to_string());
                    }
                } else {
                    make_error("expected punctuation".to_string());
                }
            }
            State::CheckIfTableOrFieldKeyWord => {
                if let TokenTree::Ident(ident) = &token {
                    if current_field != Field::default() {
                        current_table.fields.push(current_field.clone());
                        current_field = Field::default();
                    };

                    match ident.to_string().as_str() {
                        TABLE => {
                            if current_table != Table::default() {
                                schema.tables.push(current_table.clone());
                                current_table = Table::default();
                            };
                            state = State::StoreTableId
                        }
                        FIELD => state = State::StoreFieldId,
                        _ => make_error(format!("expected '{}' or '{}'", TABLE, FIELD)),
                    }
                } else {
                    make_error("expected identifier".to_string());
                }
            }
            State::CheckIfArrayOrEnumOrRecordKeyWordOrType => {
                if let TokenTree::Ident(ident) = &token {
                    match ident.to_string().as_str() {
                        RECORD => {
                            current_field.data_type = DataType::Record(vec![]);
                            state = State::ProcessOpenRecord;
                        }
                        _ => {
                            current_field.data_type = DataType::SingleType(ident.to_string());
                            state = State::CheckIfTableOrFieldKeyWord;
                        }
                    }
                } else {
                    make_error("expected identifier".to_string());
                }
            }
            State::StoreTableId => {
                if let TokenTree::Literal(_) = &token {
                    match token_to_u::<u16>(&token) {
                        Some(number) => {
                            current_table.id = number;
                            state = State::StoreTableName
                        }
                        None => make_error(format!("expected number u16 but found '{}'", token)),
                    };
                } else {
                    make_error("expected literal".to_string());
                }
            }
            State::StoreTableName => {
                if let TokenTree::Ident(ident) = &token {
                    current_table.name = ident.to_string();
                    state = State::FieldKeyWord;
                } else {
                    make_error("expected identifier".to_string());
                }
            }
            State::FieldKeyWord => {
                if let TokenTree::Ident(ident) = &token {
                    if ident.to_string().as_str() != FIELD {
                        make_error(format!("expected '{}' keyword", FIELD));
                    } else {
                        state = State::StoreFieldId;
                    }
                } else {
                    make_error("expected identifier".to_string());
                }
            }
            State::StoreFieldId => {
                if let TokenTree::Literal(_) = &token {
                    match token_to_u::<u8>(&token) {
                        Some(number) => {
                            current_field.id = number;
                            state = State::StoreFieldName
                        }
                        None => make_error(format!("expected number u8 but found '{}'", token)),
                    };
                } else {
                    make_error("expected literal".to_string());
                }
            }
            State::StoreFieldName => {
                if let TokenTree::Ident(ident) = &token {
                    current_field.name = ident.to_string();
                    state = State::CheckIfArrayOrEnumOrRecordKeyWordOrType;
                } else {
                    make_error("expected identifier".to_string());
                }
            }
        }
    }

    if current_field != Field::default() {
        current_table.fields.push(current_field.clone());
    };
    if current_table != Table::default() {
        schema.tables.push(current_table.clone());
    };

    if errors.is_empty() {
        return Ok(schema);
    } else {
        return Err(errors.into());
    }
}

// fn generate_ast(input: TokenStream) -> SchemaAST {
//     let lines = split_by_line(input);

//     let mut schema = SchemaAST::default();
//     let mut table_index = 0;

//     for (i, line) in lines.iter().enumerate() {
//         let key_word_table_or_field = &line[0];
//         let id = &line[1];
//         let name = &line[2];

//         if is_ident(key_word_table_or_field, TABLE) {
//             let mut table = Table::default();

//             table.id = token_to_u::<u16>(id).unwrap();
//             table.name = name.to_string();

//             schema.tables.push(table);
//             table_index += 1;
//         } else if is_ident(key_word_table_or_field, FIELD) {
//             let forth_word = &line[3];
//             let mut data_type = forth_word;

//             let mut field = Field::default();

//             if is_ident(forth_word, RECORD) {
//                 field.is_record_id = true;
//                 data_type = &line[4];
//             }

//             field.id = token_to_u::<u8>(id).unwrap();
//             field.name = name.to_string();
//             field.data_type = data_type.to_string();

//             schema.tables[table_index - 1].fields.push(field);
//         }
//     }

//     return schema;
// }

use std::collections::HashSet;

fn check_ast(schema: &SchemaAST) {
    let mut table_names = HashSet::new();
    let mut table_ids = HashSet::new();

    for table in &schema.tables {
        // Check table name
        if !table_names.insert(&table.name) {
            panic!("Duplicate table name: '{}'", table.name);
        }

        // Check table ID
        if !table_ids.insert(table.id) {
            panic!("Duplicate table ID: {}", table.id);
        }

        // Check fields in this table
        let mut field_names = HashSet::new();
        let mut field_ids = HashSet::new();

        for field in &table.fields {
            // Check field name within table
            if !field_names.insert(&field.name) {
                panic!(
                    "Duplicate field name '{}' in table '{}'",
                    field.name, table.name
                );
            }

            // Check field ID within table
            if !field_ids.insert(field.id) {
                panic!("Duplicate field ID {} in table '{}'", field.id, table.name);
            }
        }
    }

    for table in &schema.tables {
        for field in &table.fields {
            match field.data_type {
                DataType::SingleType(_) => {}
                DataType::Record(ref e) => {
                    for table_name in e {
                        if !table_names.contains(&table_name) {
                            panic!(
                                "this table is not exist '{}' , see field '{}' in table '{}'",
                                table_name, field.name, table.name
                            );
                        }
                    }
                }
            }
        }
    }
}

fn make_graph(schema: &SchemaAST) {}

fn generate_rust_code(schema: &SchemaAST) -> proc_macro2::TokenStream {
    let mut code = proc_macro2::TokenStream::new();

    code.extend(quote! {
            use serde::{Deserialize, Serialize};
            use derive_more::From;

            #[derive(Debug, Deserialize, Serialize, Clone)]
            pub struct TableAndField {
                pub table: u16,
                pub field: u8,
            }

            #[derive(Debug, Deserialize, Serialize, Clone)]
            pub struct TableAndRecord {
                pub table: u16,
                pub record: u64,
            }
        });

    for table in &schema.tables {
        let const_table_id = to_upper_case(&format!("table_{}_id", &table.name));
        let const_table_name = to_upper_case(&format!("table_{}_name", &table.name));
        let type_alias_for_table_and_record = to_pascal_case(&format!("table_{}", &table.name));
        let type_alias_for_record_id = to_pascal_case(&format!("table_{}_record_id", &table.name));

        let type_alias_for_table_and_record_ident = Ident::new(
            type_alias_for_table_and_record.as_str(),
            proc_macro2::Span::call_site(),
        );
        let type_alias_for_record_id_ident = Ident::new(
            type_alias_for_record_id.as_str(),
            proc_macro2::Span::call_site(),
        );
        let const_table_id_ident =
            Ident::new(const_table_id.as_str(), proc_macro2::Span::call_site());
        let const_table_name_ident =
            Ident::new(const_table_name.as_str(), proc_macro2::Span::call_site());

        let name_of_the_table = table.name.as_str();
        let table_id = table.id;

        let table_code = quote! {
            pub type #type_alias_for_record_id_ident = u64;
            pub type #type_alias_for_table_and_record_ident = TableAndRecord;
            pub const #const_table_id_ident : u16 = #table_id;
            pub const #const_table_name_ident : &str = #name_of_the_table;
        };
        code.extend(table_code);

        for field in &table.fields {
            let const_field_id =
                to_upper_case(&format!("table_{}_field_{}_id", &table.name, &field.name));
            let const_field_name =
                to_upper_case(&format!("table_{}_field_{}_name", &table.name, &field.name));
            let type_field_name =
                to_pascal_case(&format!("table_{}_field_{}", &table.name, &field.name));

            let type_field_name_ident =
                Ident::new(type_field_name.as_str(), proc_macro2::Span::call_site());
            let const_field_id_ident =
                Ident::new(const_field_id.as_str(), proc_macro2::Span::call_site());
            let const_field_name_ident =
                Ident::new(const_field_name.as_str(), proc_macro2::Span::call_site());

            let field_type_ident: Ident;
            match &field.data_type {
                DataType::SingleType(the_type) => {
                    field_type_ident =
                        Ident::new(&the_type.as_str(), proc_macro2::Span::call_site());
                }
                DataType::Record(_) => {
                    field_type_ident = Ident::new("TableAndRecord", proc_macro2::Span::call_site());
                }
            }

            let name_of_the_field = field.name.as_str();
            let field_id = field.id;

            let field_code = quote! {
                pub type #type_field_name_ident = #field_type_ident;
                pub const #const_field_id_ident : TableAndField = TableAndField{table: #table_id, field: #field_id};
                pub const #const_field_name_ident : &str = #name_of_the_field;
            };
            code.extend(field_code);
        }
    }

    return code;
}

// Helper: Convert to UPPER_CASE
fn to_upper_case(input: &String) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        result.push(c.to_uppercase().next().unwrap());
    }

    result
}

// Helper: Convert to PascalCase
fn to_pascal_case(input: &String) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in input.chars() {
        if c == '_' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

fn token_to_u<T: FromStr>(token: &TokenTree) -> Option<T> {
    match token {
        TokenTree::Literal(lit) => {
            let s = lit.to_string();
            // Remove underscores like 1_000
            let clean = s.replace('_', "");
            clean.parse::<T>().ok()
        }
        _ => None,
    }
}

fn is_ident(token: &TokenTree, name: &str) -> bool {
    if let TokenTree::Ident(ident) = token {
        ident.to_string() == name
    } else {
        false
    }
}

fn split_by_line(tokens: TokenStream) -> Vec<Vec<TokenTree>> {
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut current_line_num = 0;
    let mut first_token = true;

    for token in tokens {
        let line = token.span().start().line();

        if first_token {
            current_line_num = line;
            first_token = false;
        }

        if line != current_line_num {
            lines.push(current_line);
            current_line = Vec::new();
            current_line_num = line;
        }

        current_line.push(token);
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
