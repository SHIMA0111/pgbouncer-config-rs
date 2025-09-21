use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(Expression)]
pub fn expression_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;

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
        }
    };

    generated.into()
}