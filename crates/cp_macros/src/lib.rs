use proc_macro::TokenStream;

use darling::FromMeta;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

#[derive(Debug, FromMeta)]
struct TestArgs {
    log: Option<String>,
    #[darling(default)]
    write_log_file: bool,
}

#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = match TestArgs::from_list(&parse_macro_input!(attr as AttributeArgs)) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let write_log_file = if args.write_log_file {
        quote! {}
    } else {
        quote! {
            std::env::set_var("NOT_WRITE_LOG_FILE", "");
        }
    };

    let log_level = args.log.unwrap_or_else(|| String::from("INFO"));
    let log_level = quote! {
        std::env::set_var("LOG_LEVEL", #log_level);
    };

    let mut item = parse_macro_input!(item as ItemFn);

    let block = item.block;
    item.block = syn::parse2(quote! {
        {
            #write_log_file
            #log_level
            let fut = crate::environment::init_environment()?;
            let ret = #block;
            fut.await?;
            ret
        }
    })
    .expect("parsing failure");

    TokenStream::from(quote! {
        #[::tokio::test]
        #item
    })
}