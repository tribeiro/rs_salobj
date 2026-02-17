use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::env;
use std::fs;

#[proc_macro]
pub fn ts_xml_define(input: TokenStream) -> TokenStream {
    // Parse input identifier (e.g., Test)
    let name = syn::parse_macro_input!(input as syn::Ident);

    // Get environment variable
    let dir = env::var("TS_XML_DIR").expect("TS_XML_DIR environment variable must be set");

    // Build XML path
    let telemetry_path = format!("{}/{}/{}_Telemetry.xml", dir, name, name);
    let event_path = format!("{}/{}/{}_Events.xml", dir, name, name);
    let command_path = format!("{}/{}/{}_Commands.xml", dir, name, name);
    let generics_path = format!("{}/SALGenerics.xml", dir);

    // Collect all structs
    let structs: Vec<TokenStream2> = expand_topics(&telemetry_path, None)
        .into_iter()
        .chain(
            expand_topics(&event_path, None).into_iter().chain(
                expand_topics(&command_path, None)
                    .into_iter()
                    .chain(expand_topics(&generics_path, Some(&format!("{name}")))),
            ),
        )
        .collect();

    let expanded = quote! {
        #(#structs)*
    };

    expanded.into()
}

fn expand_topics(topic_file_path: &str, replace_generic_with: Option<&str>) -> Vec<TokenStream2> {
    let xml_content = fs::read_to_string(topic_file_path)
        .unwrap_or_else(|_| panic!("Failed to read XML file: {}", topic_file_path));

    // Parse XML
    let doc = roxmltree::Document::parse(&xml_content)
        .unwrap_or_else(|e| panic!("Error parsing XML file {}: {}", topic_file_path, e));

    // Find all <SALTelemetry> nodes
    let topics: Vec<_> = doc
        .descendants()
        .filter(|n| {
            n.has_tag_name("SALTelemetry")
                || n.has_tag_name("SALEvent")
                || n.has_tag_name("SALCommand")
        })
        .collect();

    // Collect all structs
    let mut structs = Vec::new();

    for topic_data in topics {
        let topic = topic_data
            .descendants()
            .find(|n| n.has_tag_name("EFDB_Topic"))
            .and_then(|n| n.text())
            .map(|n| {
                if let Some(component_name) = replace_generic_with {
                    n.to_string().replace("SALGeneric", component_name)
                } else {
                    n.to_string()
                }
            })
            .unwrap_or_else(|| panic!("Missing EFDB_Topic in {}", topic_file_path))
            .to_case(Case::Pascal);

        let topic_ident = format_ident!("{}", topic);

        let mut fields = Vec::new();

        for item in topic_data.children().filter(|n| n.has_tag_name("item")) {
            let name = item
                .descendants()
                .find(|n| n.has_tag_name("EFDB_Name"))
                .and_then(|n| n.text())
                .unwrap()
                .to_string();

            let idl_type = item
                .descendants()
                .find(|n| n.has_tag_name("IDL_Type"))
                .and_then(|n| n.text())
                .unwrap()
                .trim()
                .to_string();

            let rust_type = match idl_type.as_str() {
                "boolean" => quote! { bool },
                "byte" => quote! { i8 },
                "short" => quote! { i16 },
                "int" => quote! { i32 },
                "long" => quote! { i64 },
                "long long" => quote! { i64 },
                "unsigned short" => quote! { u16 },
                "unsigned int" => quote! { u32 },
                "unsigned long" => quote! { u64 },
                "float" => quote! { f32 },
                "double" => quote! { f64 },
                "string" => quote! { String },
                _ => panic!("Unknown IDL_Type: {}", idl_type),
            };

            let count = item
                .descendants()
                .find(|n| n.has_tag_name("Count"))
                .and_then(|n| n.text())
                .and_then(|t| t.parse::<usize>().ok())
                .unwrap_or(1);

            let fname = format_ident!("{}", name);

            let field_type = if count > 1 {
                quote! { [#rust_type; #count] }
            } else {
                quote! { #rust_type }
            };

            fields.push(quote! { pub #fname: #field_type });
        }

        fields.push(quote! { private_origin: i32 });
        fields.push(quote! { private_identity: String });
        fields.push(quote! { private_seqNum: i32 });
        fields.push(quote! { private_rcvStamp: f64 });
        fields.push(quote! { private_sndStamp: f64 });
        fields.push(quote! { pub salIndex: i32 });
        fields.push(quote! { private_efdStamp: f64 });
        fields.push(quote! { private_kafkaStamp: f64});
        fields.push(quote! { private_revCode: String });

        structs.push(quote! {
            #[derive(Debug, Clone, Default, Deserialize)]
            pub struct #topic_ident {
                #(#fields),*
            }
        });
    }
    structs
}
