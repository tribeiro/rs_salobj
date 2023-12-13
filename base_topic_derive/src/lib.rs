use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, DeriveInput};

fn impl_base_topic_trait(ast: DeriveInput) -> TokenStream {
    // generate struct identifier
    let ident = ast.ident; // struct identifies, basically the name of the struct
    let ident_str = ident.to_string();

    let field_idents: Vec<syn::Ident> = match ast.data {
        syn::Data::Struct(data) => data.fields.into_iter().filter_map(|f| f.ident).collect(),
        syn::Data::Enum(_) => panic!("Enums are not supported by BaseSALTopic."),
        syn::Data::Union(_) => panic!("Unions are not supported by BaseSALTopic."),
    };

    let field_idents_strs: Vec<String> = field_idents.iter().map(|i| i.to_string()).collect();

    // generate impl
    quote::quote!(
        impl BaseSALTopic for #ident {
            fn get_name(&self) -> &'static str {
                #ident_str
            }
            fn field_names(&self) -> Vec<&'static str> {
                vec![#(#field_idents_strs),*]
            }
            fn get_private_origin(&self) -> i64 {
                self.private_origin
            }
            fn get_private_identity(&self) -> &str {
                &self.private_identity
            }
            fn get_private_seq_num(&self) -> i64 {
                self.private_seq_num
            }
            fn get_private_rcv_stamp(&self) -> f64 {
                self.private_rcv_stamp
            }
            fn get_sal_index(&self) -> i64 {
                self.sal_index
            }
            fn set_private_snd_stamp(&mut self, value: f64) {
                self.private_snd_stamp = value;
            }
            fn set_private_efd_stamp(&mut self, value: f64) {
                self.private_efd_stamp = value;
            }
            fn set_private_kafka_stamp(&mut self, value: f64) {
                self.private_kafka_stamp = value;
            }
            fn set_private_origin(&mut self, value: i64) {
                self.private_origin = value;
            }
            fn set_private_identity(&mut self, value: &str) {
                self.private_identity = value.to_owned();
            }
            fn set_private_rev_code(&mut self, value: &str) {
                self.private_rev_code = value.to_owned();
            }
            fn set_private_seq_num(&mut self, value: i64) {
                self.private_seq_num = value;
            }
            fn set_private_rcv_stamp(&mut self, value: f64) {
                self.private_rcv_stamp = value;
            }
            fn set_sal_index(&mut self, value: i64) {
                self.sal_index = value;
            }
        }
    )
    .into()
}

#[proc_macro_derive(BaseSALTopic)]
pub fn base_topic_derive_macro(item: TokenStream) -> TokenStream {
    // parse
    let ast: DeriveInput = syn::parse(item).unwrap();

    // generate
    impl_base_topic_trait(ast)
}

#[proc_macro_attribute]
pub fn add_sal_topic_fields(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            match &mut struct_data.fields {
                syn::Fields::Named(fields) => {
                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! { private_origin: i64 })
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
                                private_seq_num: i64
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
                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! {
                                #[serde(rename = "salIndex", default = "get_default_sal_index")]
                                sal_index: i64
                            })
                            .unwrap(),
                    );
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
                _ => (),
            }

            return quote! {
                #ast
            }
            .into();
        }
        _ => panic!("`add_sal_topic_fields` has to be used with structs "),
    }
}
