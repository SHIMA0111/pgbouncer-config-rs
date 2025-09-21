use proc_macro::TokenStream;
use heck::ToKebabCase;
use quote::quote;
use syn::{parse_macro_input, LitStr};
use syn::parse::ParseStream;

#[proc_macro_derive(Expression, attributes(expression))]
pub fn expression_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;

    let section_name_override = ast.attrs.iter()
        .find(|attr| attr.path().is_ident("expression"))
        .map(|attr| attr.parse_args::<SectionNameAttr>().unwrap().section_name);

    let section_name = match section_name_override {
        Some(attr) => quote! { #attr },
        None => {
            let default_name = name.to_string().to_kebab_case();
            quote! { #default_name }
        }
    };

    let generated = quote! {
        #[typetag::serde]
        impl pgbouncer_config::pgbouncer_config::Expression for #name {
            fn expr(&self) -> pgbouncer_config::error::Result<String> {
                use pgbouncer_config::__private::ExpressionDefault;

                let section_name = self.section_name();
                let mut buffer = String::new();
                buffer.push_str(format!("[{}]\n", section_name).as_str());
                buffer.push_str(self.to_expr_default()?.as_str());
                Ok(buffer)
            }

            fn section_name(&self) -> &'static str {
                #section_name
            }
        }
    };

    generated.into()
}

struct SectionNameAttr {
    section_name: LitStr
}

impl syn::parse::Parse for SectionNameAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _key: syn::Ident = input.parse()?;
        let _eq: syn::Token![=] = input.parse()?;
        let section_name: LitStr = input.parse()?;
        Ok(SectionNameAttr {
            section_name
        })
    }
}