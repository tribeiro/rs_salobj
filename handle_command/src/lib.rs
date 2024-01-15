extern crate proc_macro;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, LitStr, Result, Token};

#[proc_macro]
pub fn handle_command(input: TokenStream) -> TokenStream {
    let items = parse_macro_input!(input as MyMacroInput);
    let output = expand_my_macro(items);
    TokenStream::from(output)
}

struct MyMacroInput {
    items: Vec<LitStr>,
}

impl syn::parse::Parse for MyMacroInput {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut items = Vec::new();

        while !input.is_empty() {
            let item: LitStr = input.parse()?;
            items.push(item);

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(MyMacroInput { items })
    }
}

fn expand_my_macro(input: MyMacroInput) -> TokenStream2 {
    let items = &input.items;

    let code = items.iter().map(|item| {
        let item_value = &item.value();
        let method_name = &item.value().to_case(Case::Snake);
        // let concatenated = format!("self.do_{}", item_value);
        // let varname = syn::Ident::new(&concatenated, Span::call_site());
        let varname = format_ident! {"do_{}", method_name};

        quote! {
            else if data.name == format!("command_{}", #item_value) {
                let (command_ack, ack_channel) = self.#varname(&data, ack_channel).await?;
                let _ = ack_channel.send(command_ack).await;
            }
        }
    });

    quote! {
        if data.name == "command_exitControl" {
            let (command_ack, ack_channel) = self.do_exit_control(&data, ack_channel).await?;
            let command_ack_is_good = command_ack.is_good();
            let _ = ack_channel.send(command_ack).await;
            sleep(Duration::from_secs(1)).await;
            if command_ack_is_good {
                break;
            }
        }
        #(#code)*
            else {
            let any = from_value::<ExitControl>(&data.data).unwrap();
            let command_ack = CommandAck::make_failed(
                any,
                1,
                &format!("Command {} not implemented.", data.name),
            );
            let _ = ack_channel.send(command_ack).await;
            continue;
        }

    }
}
