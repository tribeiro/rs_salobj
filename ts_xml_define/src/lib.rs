use proc_macro::TokenStream;
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
    let xml_content = fs::read_to_string(&telemetry_path)
        .unwrap_or_else(|_| panic!("Failed to read XML file: {}", telemetry_path));

    // Parse XML
    let doc = roxmltree::Document::parse(&xml_content)
        .unwrap_or_else(|e| panic!("Error parsing XML file {}: {}", telemetry_path, e));

    // Find all <SALTelemetry> nodes
    let telemetries: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("SALTelemetry"))
        .collect();

    // Collect all structs
    let mut structs = Vec::new();

    for telemetry in telemetries {
        let topic = telemetry
            .descendants()
            .find(|n| n.has_tag_name("EFDB_Topic"))
            .and_then(|n| n.text())
            .unwrap_or_else(|| panic!("Missing EFDB_Topic in {}", telemetry_path))
            .to_string();

        let topic_ident = format_ident!("{}", topic);

        let mut fields = Vec::new();

        for item in telemetry.children().filter(|n| n.has_tag_name("item")) {
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
            #[derive(Debug, Clone, Default)]
            pub struct #topic_ident {
                #(#fields),*
            }
        });
    }

    let expanded = quote! {
        #(#structs)*
    };

    expanded.into()
}
