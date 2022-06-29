use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Generics, Ident};

macro_rules! bail {
    ($span:expr, $fmt:expr $(,$args:expr)*) => {
        return Err(::syn::Error::new(
            $span, format!($fmt $(,$args)*)
        ))
    }
}

#[proc_macro_derive(IntoRow)]
pub fn my_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input = match IntoRowInput::from_derive(input) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };

    input.to_token_stream().into()
}

struct IntoRowInput {
    name: Ident,
    generics: Generics,
    fields: IntoRowFields,
}

impl IntoRowInput {
    fn from_derive(input: DeriveInput) -> Result<Self, syn::Error> {
        let struct_ = match input.data {
            Data::Struct(v) => v,
            Data::Enum(e) => {
                return Err(syn::Error::new(
                    e.enum_token.span,
                    "can only derive this trait on structs",
                ))
            }
            Data::Union(u) => {
                return Err(syn::Error::new(
                    u.union_token.span,
                    "can only derive this trait on structs",
                ))
            }
        };

        let fields = match struct_.fields {
            Fields::Named(fields) => {
                if fields.named.is_empty() {
                    bail!(
                        struct_.struct_token.span,
                        "no data to display for zero-sized types"
                    );
                }
                IntoRowFields::Named(
                    fields
                        .named
                        .into_iter()
                        .map(|field| (field.ident.unwrap()))
                        .collect(),
                )
            }
            Fields::Unnamed(fields) => {
                if fields.unnamed.is_empty() {
                    bail!(
                        struct_.struct_token.span,
                        "no data to display for zero-sized types"
                    );
                }
                IntoRowFields::Unnamed(fields.unnamed.len())
            }
            Fields::Unit => bail!(
                struct_.struct_token.span,
                "no data to display for zero-sized types"
            ),
        };

        Ok(Self {
            name: input.ident,
            fields,
            generics: input.generics,
        })
    }
}

impl ToTokens for IntoRowInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let (g_impl, g_type, g_where) = self.generics.split_for_impl();
        let headers = self.fields.headers();
        let into_row = self.fields.into_row();

        tokens.extend(quote! {
            impl #g_impl ::term_data_table::IntoRow for #name #g_type #g_where {
                fn headers(&self) -> ::term_data_table::Row {
                    #headers
                }

                fn into_row(&self) -> ::term_data_table::Row {
                    #into_row
                }
            }
        })
    }
}

enum IntoRowFields {
    Named(Vec<Ident>),
    Unnamed(usize),
}

impl IntoRowFields {
    fn headers(&self) -> TokenStream {
        match self {
            Self::Named(idents) => {
                let idents = idents.into_iter().map(|ident| ident.to_string());
                quote! {
                    ::term_data_table::Row::new()
                    #(
                        .with_cell(::term_data_table::Cell::from(#idents))
                    )*
                }
            }
            Self::Unnamed(count) => {
                let idents = (0..*count).map(|idx| idx.to_string());
                quote! {
                    ::term_data_table::Row::new()
                    #(
                        .with_cell(::term_data_table::Cell::from(#idents))
                    )*
                }
            }
        }
    }

    fn into_row(&self) -> TokenStream {
        match self {
            Self::Named(idents) => {
                let idents = idents.into_iter();
                quote! {
                    ::term_data_table::Row::new()
                    #(
                        .with_cell(::term_data_table::Cell::from(self.#idents.to_string()))
                    )*
                }
            }
            Self::Unnamed(count) => {
                let idents = 0..*count;
                quote! {
                    ::term_data_table::Row::new()
                    #(
                        .with_cell(::term_data_table::Cell::from(self.#idents.to_string()))
                    )*
                }
            }
        }
    }
}
