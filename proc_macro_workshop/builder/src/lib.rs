#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let input_ident = &input.ident;
    // let ident = Ident::new(&format!("{}Builder", input.ident), Span::call_site());
    let builder_ident = quote::format_ident!("{}Builder", &input.ident);

    let named_fields = get_named_fields(&input.data);

    let ts_input_builder = create_builder_ts(&builder_ident, named_fields);
    let ts_builder_struct = create_struct_ts(&builder_ident, named_fields);
    let ts_builder_setters = create_setters_ts(named_fields);
    let ts_builder_build = create_build_ts(input_ident, named_fields);


    let expanded = quote::quote! {
        impl #input_ident {
            #ts_input_builder
        }

        #ts_builder_struct

        impl #builder_ident {
            #ts_builder_setters
            #ts_builder_build
        }
    };

    proc_macro::TokenStream::from(expanded)
}

/// Get named fields for the provided struct
fn get_named_fields(data: &syn::Data) -> &syn::FieldsNamed {

    match *data {
        syn::Data::Struct(ref d) => {
            match d.fields {
                syn::Fields::Named(ref fields) => {
                    fields
                }
                syn::Fields::Unnamed(_) | syn::Fields::Unit => {
                    unimplemented!()
                }
            }

        },
        syn::Data::Enum(_) | syn::Data::Union(_) => {
            unimplemented!()
        },
    }
}

/// Create TokenStream for build method
fn create_builder_ts(builder_ident: &proc_macro2::Ident, named_fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    let recurse = named_fields.named.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();

        use syn::spanned::Spanned;
        quote::quote_spanned!(f.span() =>
            #name: None,
        )
    });

    quote::quote! {
        pub fn builder() -> #builder_ident {
            #builder_ident {
                #(#recurse)*
            }
        }
    }
}

/// Create TokenStream for build method
fn create_struct_ts(builder_ident: &proc_macro2::Ident, named_fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    let recurse = named_fields.named.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();

        use syn::spanned::Spanned;
        let ty = is_filed_optional(f).unwrap_or(&f.ty);
        quote::quote_spanned!(f.span() =>
            #name: Option<#ty>,
        )
    });

    quote::quote! {
        pub struct #builder_ident {
            #(#recurse)*
        }
    }
}

/// Generate TokenStream for setter methods
fn create_setters_ts(named_fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
        let mut q = quote::quote!();

        named_fields.named.iter().for_each(|f| {
            let name = f.ident.as_ref().unwrap();
            let ty = is_filed_optional(f).unwrap_or(&f.ty);

            q.extend(
                quote::quote! {
                    pub fn #name (&mut self, #name: #ty) -> &mut Self {
                        self.#name = Some(#name);
                        self
                    }
                });
        });

        q
}

/// Create TokenStream for build method
fn create_build_ts(input_ident: &proc_macro2::Ident, named_fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    let recurse = named_fields.named.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();

        use syn::spanned::Spanned;

        if is_filed_optional(f).is_some() {
            // just clone the optional value
            quote::quote_spanned!(f.span() =>
                #name: self.#name.to_owned(),
            )
        } else {
            // return error if field is none, otherwise clone
            quote::quote_spanned!(f.span() =>
                #name: self.#name.as_ref()
                                 .ok_or::<Box<dyn std::error::Error>>(String::from("ERROR").into())?
                                 .to_owned(),
            )
        }
    });

    quote::quote! {

        pub fn build(&mut self) -> Result<#input_ident, Box<dyn std::error::Error>> {
            Ok(#input_ident {
                #( #recurse )*
            })
        }

    }
}

/// Checks weather a given field is optional
fn is_filed_optional(field: &syn::Field) -> Option<&syn::Type> {

    if let syn::Type::Path(ref path) = field.ty {
        for s in path.path.segments.iter() {
            if s.ident != "Option" { continue; }

            if let syn::PathArguments::AngleBracketed(ref bracket_args) = s.arguments {

                for a in &bracket_args.args {
                    if let syn::GenericArgument::Type(ref t) = a {
                        return Some(t);
                    }
                }

            }
        }
    }
    None
}
