use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, DeriveInput};

fn impl_base_topic_trait(ast: DeriveInput, indexed: bool) -> TokenStream {
    // generate struct identifier
    let ident = ast.ident; // struct identifies, basically the name of the struct
    let ident_str = ident.to_string();

    let field_idents: Vec<syn::Ident> = match ast.data {
        syn::Data::Struct(data) => data.fields.into_iter().filter_map(|f| f.ident).collect(),
        syn::Data::Enum(_) => panic!("Enums are not supported by BaseSALTopic."),
        syn::Data::Union(_) => panic!("Unions are not supported by BaseSALTopic."),
    };

    let field_idents_strs: Vec<String> = field_idents.iter().map(|i| i.to_string()).collect();

    let sal_index_methods = if indexed {
        quote::quote! {
            fn get_sal_index(&self) -> i32 {
                self.sal_index
            }
            fn with_sal_index(mut self, value: i32) -> Self {
                self.sal_index = value;
                self
            }
        }
    } else {
        quote::quote! {
            fn get_sal_index(&self) -> i32 {
                0
            }
            fn with_sal_index(self, _value: i32) -> Self {
                self
            }
        }
    };

    // generate impl
    quote::quote!(
        impl ::salobj::topics::base_sal_topic::BaseSALTopic for #ident {
            fn get_name(&self) -> &'static str {
                #ident_str
            }
            fn field_names(&self) -> Vec<&'static str> {
                vec![#(#field_idents_strs),*]
            }
            fn get_private_origin(&self) -> i32 {
                self.private_origin
            }
            fn get_private_identity(&self) -> &str {
                &self.private_identity
            }
            fn get_private_seq_num(&self) -> i32 {
                self.private_seq_num
            }
            fn get_private_rcv_stamp(&self) -> f64 {
                self.private_rcv_stamp
            }
            fn with_timestamps(mut self) -> Self {
                let timestamp = Utc::now().timestamp_micros() as f64 * 1e-6;
                self.private_snd_stamp = timestamp;
                self.private_efd_stamp = timestamp;
                self.private_kafka_stamp = timestamp;
                self
            }
            fn with_private_origin(mut self, value: i32) -> Self {
                self.private_origin = value;
                self
            }
            fn with_private_identity(mut self, value: &str) -> Self {
                self.private_identity = value.to_owned();
                self
            }
            fn with_private_rev_code(mut self, value: &str) -> Self {
                self.private_rev_code = value.to_owned();
                self
            }
            fn with_private_seq_num(mut self, value: i32) -> Self {
                self.private_seq_num = value;
                self
            }
            #sal_index_methods
        }
    )
    .into()
}

#[proc_macro_derive(BaseSALTopic, attributes(base_sal_topic))]
pub fn base_topic_derive_macro(item: TokenStream) -> TokenStream {
    // parse
    let ast: DeriveInput = syn::parse(item).unwrap();
    let indexed: bool = {
        let mut indexed = false;

        for attr in &ast.attrs {
            if attr.path().is_ident("base_sal_topic") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("indexed") {
                        indexed = true;
                        Ok(())
                    } else {
                        Err(meta.error("unsupported property in base_sal_topic"))
                    }
                })
                .expect("Failed to parse #[base_sal_topic] attribute");
            }
        }
        indexed
    };

    // generate
    impl_base_topic_trait(ast, indexed)
}

#[proc_macro_attribute]
pub fn add_sal_topic_fields(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse `#[add_sal_topic_fields]` / `#[add_sal_topic_fields(indexed)]`.
    let mut indexed = false;
    let args_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("indexed") {
            indexed = true;
            Ok(())
        } else {
            Err(meta.error("unsupported property in add_sal_topic_fields"))
        }
    });
    parse_macro_input!(args with args_parser);

    let mut ast = parse_macro_input!(input as DeriveInput);

    // Forward the switch to the `BaseSALTopic` derive, which cannot see the
    // arguments of this attribute macro.
    if indexed {
        ast.attrs
            .push(syn::parse_quote!(#[base_sal_topic(indexed)]));
    }

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            if let syn::Fields::Named(fields) = &mut struct_data.fields {
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! { private_origin: i32 })
                        .unwrap(),
                );
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! { private_identity: String })
                        .unwrap(),
                );
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            #[serde(rename = "private_seqNum")]
                            private_seq_num: i32
                        })
                        .unwrap(),
                );
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            #[serde(rename = "private_rcvStamp")]
                            private_rcv_stamp: f64
                        })
                        .unwrap(),
                );
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            #[serde(rename = "private_sndStamp")]
                            private_snd_stamp: f64
                        })
                        .unwrap(),
                );
                if indexed {
                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! {
                                #[serde(rename = "salIndex", default = "get_default_sal_index")]
                                sal_index: i32
                            })
                            .unwrap(),
                    );
                }
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            #[serde(rename = "private_efdStamp")]
                            private_efd_stamp: f64
                        })
                        .unwrap(),
                );
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            #[serde(rename = "private_kafkaStamp")]
                            private_kafka_stamp: f64
                        })
                        .unwrap(),
                );
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            #[serde(rename = "private_revCode")]
                            private_rev_code: String
                        })
                        .unwrap(),
                );
            }

            quote! {
                #ast
            }
            .into()
        }
        _ => panic!("`add_sal_topic_fields` has to be used with structs "),
    }
}
