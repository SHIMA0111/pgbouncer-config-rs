use proc_macro::TokenStream;
use std::collections::HashSet;
use darling::{FromDeriveInput};
use darling::util::Override;
use heck::ToKebabCase;
use quote::quote;
use syn::parse_macro_input;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(expression), supports(struct_named))]
struct ExpressionOpts {
    ident: syn::Ident,
    data: darling::ast::Data<(), syn::Field>,
    #[darling(default)]
    section_name: Override<String>,
    #[darling(default)]
    template: Override<String>,
}

#[proc_macro_derive(Expression, attributes(expression))]
pub fn expression_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    let opts = match ExpressionOpts::from_derive_input(&ast) {
        Ok(opts) => opts,
        Err(e) => return e.write_errors().into(),
    };

    let struct_name = &opts.ident;

    // --- Solve the section_name between user definition or default ---
    let section_name = match &opts.section_name {
        Override::Explicit(name) => quote! { #name },
        Override::Inherit => {
            let default_name = struct_name.to_string().to_kebab_case();
            quote! { #default_name }
        }
    };

    // --- Solve the template between user definition or default ---
    let template_str = match &opts.template {
        Override::Explicit(template) => template.to_string(),
        Override::Inherit => {
            if let syn::Data::Struct(data_struct) = &ast.data {
                if let syn::Fields::Named(fields_named) = &data_struct.fields {
                    fields_named.named.iter()
                        .filter_map(|field| field.ident.as_ref())
                        .map(|ident| {
                            let name = ident.to_string();
                            format!("{0} = {{{0}}}", name)
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        }
    };

    let segments = parse_template(&template_str).unwrap();

    // --- Prepare the to_template_string code generation ---
    let format_string = segments
        .iter()
        .map(|seg| match seg {
            TemplateSegment::Literal(s) => s.replace("{", "{{").replace("}", "}}"),
            TemplateSegment::Placeholder(_) => "{}".to_string()
        }).collect::<String>();

    let format_args = segments
        .iter()
        .filter_map(|seg| match seg {
            TemplateSegment::Placeholder(name) => {
                let field_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                Some(quote! { &self.#field_ident })
            },
            TemplateSegment::Literal(_) => None,
        });

    // --- Prepare from_template_string code generation ---
    let all_fields = if let darling::ast::Data::Struct(data_struct) = &opts.data {
        &data_struct.fields
    } else {
        // darling limits the support struct only named_struct so this branch never reachable.
        unreachable!();
    };

    // Pre validation placeholders
    let placeholder_names: HashSet<String> = segments.iter().filter_map(|seg| {
        if let TemplateSegment::Placeholder(name) = seg { Some(name.trim().to_string()) } else { None }
    }).collect();

    let all_field_names: HashSet<String> = all_fields.iter()
        .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
        .collect();

    for name in &placeholder_names {
        if !all_field_names.contains(name) {
            let error = syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Placeholder '{}' does not match any field", name)
            );
            return error.to_compile_error().into();
        }
    }

    // Build parser chain
    let mut parsers  = segments.iter().peekable();
    let mut full_parser = quote! { chumsky::prelude::empty() };
    let mut placeholder_count = 0;

    while let Some(segment) = parsers.next() {
        match segment {
            TemplateSegment::Literal(lit) => {
                if placeholder_count == 0 {
                    full_parser = quote! { chumsky::prelude::just(#lit).ignored() };
                } else {
                    full_parser = quote! { #full_parser.then_ignore(chumsky::prelude::just(#lit)) };
                }
            },
            TemplateSegment::Placeholder(name) => {
                let name_ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                let field = all_fields.iter().find(|f| f.ident.as_ref() == Some(&name_ident))
                    .expect("Template placeholder does not match any struct field");
                let field_type = &field.ty;
                let field_parser = generate_field_parser(&name_ident, field_type, parsers.peek().cloned());
                if placeholder_count == 0 {
                    full_parser = field_parser;
                } else {
                    full_parser = quote! { #full_parser.then(#field_parser) };
                }
                placeholder_count += 1;
            }
        }
    }

    full_parser = quote! { #full_parser.then_ignore(chumsky::prelude::end()) };

    let field_names: Vec<_> = segments.iter().filter_map(|s| match s {
        TemplateSegment::Placeholder(n) => Some(syn::Ident::new(n, proc_macro2::Span::call_site())),
        _ => None,
    }).collect();

    let tuple_pattern = if !field_names.is_empty() {
        // Surround tuple without the first content
        if field_names.len() > 1 {
            let first = &field_names[0];
            let rest: Vec<_> = field_names.iter().skip(1).collect();
            let mut inner_pattern = quote! { #(#rest),* };
            for _ in 1..rest.len() {
                inner_pattern = quote! { (#inner_pattern) };
            }
            quote! { (#first, #inner_pattern) }
        } else {
            quote! { #(#field_names),* }
        }
    } else {
        quote! {_}
    };

    let struct_constructor = quote! {
        #struct_name {
            #(#field_names),*
        }
    };

    let final_parser = quote! {
        #full_parser.map(|#tuple_pattern| #struct_constructor)
    };

    // --- Generate trait bound ---
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let used_fields: Vec<&syn::Field> = all_fields.iter().filter(|field| {
        if let Some(ident) = &field.ident {
            placeholder_names.contains(&ident.to_string())
        } else {
            false
        }
    }).collect();

    let mut new_where_clause = where_clause.cloned().unwrap_or_else(|| syn::parse_quote!{ where });
    for field in used_fields {
        let field_ty = &field.ty;
        if !new_where_clause.predicates.is_empty() {
            new_where_clause.predicates.push_punct(Default::default());
        }
        new_where_clause.predicates.push(syn::parse_quote! {
            #field_ty: ::std::fmt::Display + ::std::str::FromStr
        });
    }

    let where_clause = if new_where_clause.predicates.is_empty() {
        quote! {}
    } else {
        quote! { #new_where_clause }
    };

    // --- Generate the final code ---
    let generated = quote! {
        #[typetag::serde]
        impl #impl_generics pgbouncer_config::pgbouncer_config::Expression for #struct_name #ty_generics #where_clause {
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

            fn to_template_string(&self) -> String {
                format!(#format_string, #(#format_args),*)
            }

            fn from_template_string(s: &str) -> Result<Self, PgBouncerError> {
                use chumsky::Parser;

                let parser = #final_parser;

                match parser.parse(s) {
                    Ok(value) => Ok(value),
                    Err(errors) => {
                        let error_message = errors.into_iter()
                            .map(|error| error.to_string())
                            .collect::<Vec<String>>()
                            .join("\n");

                        Err(PgBouncerError::Parse(error_message))
                    }
                }
            }
        }
    };

    generated.into()
}

enum TemplateSegment<'a> {
    Literal(&'a str),
    Placeholder(&'a str),
}

fn parse_template(template: &'_ str) -> Result<Vec<TemplateSegment<'_>>, String> {
    let mut segments = Vec::new();
    let mut last_end = 0;
    let mut chars = template.char_indices().peekable();

    while let Some((idx, c)) = chars.next() {
        match c {
            '{' => {
                if let Some(&(_, next_char)) = chars.peek() {
                    if next_char == '{' {
                        chars.next();
                        continue;
                    }
                }

                if idx > last_end {
                    segments.push(TemplateSegment::Literal(&template[last_end..idx]));
                }

                let start = idx + 1;
                let end = template[start..]
                    .find('}')
                    .map(|e| start + e)
                    .ok_or_else(|| "Unmatched opening brace '{'".to_string())?;

                let placeholder = &template[start..end];
                if placeholder.contains('{') {
                    return Err("Nested braces are not supported.".to_string());
                }
                segments.push(TemplateSegment::Placeholder(placeholder.trim()));

                // Proceed index to after '}'
                last_end = idx + 1;

                while let Some((i, _)) = chars.peek().copied() {
                    if i <= end { chars.next(); } else { break; }
                }
            },
            '}' => {
                if let Some(&(_, next_char)) = chars.peek() {
                    if next_char == '}' {
                        chars.next();
                        continue;
                    }
                }
                return Err("Unmatched closing brace '}'".to_string());
            },
            _ => {}
        }
    }

    if last_end != template.len() {
        segments.push(TemplateSegment::Literal(&template[last_end..]));
    }

    Ok(segments)
}

fn generate_field_parser(
    field_name: &syn::Ident,
    field_type: &syn::Type,
    next_segment: Option<&TemplateSegment>
) -> proc_macro2::TokenStream {
    let next_literal: Option<&&str> = match next_segment {
        Some(TemplateSegment::Literal(lit)) => Some(lit),
        _ => None
    };
    let value_extractor = if let Some(next_literal) = next_literal {
        quote! {
            chumsky::prelude::any()
                .repeated()
                .take_until(chumsky::prelude::just(#next_literal).rewind())
                .to_slice()
        }
    } else {
        quote! {
            chumsky::prelude::any().repeated().to_slice()
        }
    };

    quote! {
        #value_extractor.try_map(|s: &str, span| {
            s.parse::<#field_type>()
                .map_err(|e| chumsky::error::Simple::custom(
                    span,
                    format!("Failed to parse field '{}': {}", stringify!(#field_name), e)
                ))
        })
    }
}