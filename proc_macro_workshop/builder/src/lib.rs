#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let input_ident = &input.ident;
    // let ident = Ident::new(&format!("{}Builder", input.ident), Span::call_site());
    let builder_ident = quote::format_ident!("{}Builder", &input.ident);

    let named_fields = get_named_fields(&input.data);

    let ts_input_builder = create_builder_ts(&builder_ident, named_fields);
    let ts_builder_struct = create_struct_ts(&builder_ident, named_fields);
    let ts_builder_setters: proc_macro2::TokenStream = create_setters_ts(named_fields)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into();
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
        let ty = is_field_optional(f).unwrap_or(&f.ty);
        quote::quote_spanned!(f.span() =>
            #name: std::option::Option<#ty>,
        )
    });

    quote::quote! {
        #[derive(std::fmt::Debug)]
        pub struct #builder_ident {
            #(#recurse)*
        }
    }
}

/// Generate TokenStream for setter methods
fn create_setters_ts(named_fields: &syn::FieldsNamed) -> syn::Result<proc_macro2::TokenStream> {
        use syn::{Meta, NestedMeta, Lit, Ident};

        let mut q = quote::quote!();


        for f in named_fields.named.iter() {
            let name = f.ident.as_ref().unwrap();
            let ty = is_field_optional(f).unwrap_or(&f.ty);

            let mut arg_ident = None;

            for attr in &f.attrs {

                if let Meta::List(metas) = attr.parse_meta().unwrap() {
                    if !metas.path.segments.iter().any(|s| s.ident == "builder") { continue; }

                    for nes in metas.nested.iter() {
                        if let NestedMeta::Meta(Meta::NameValue(nm)) = nes {
                            if !nm.path.segments.iter().any(|p| p.ident == "each") {
                                // invalid inert argument
                                return Err(syn::Error::new(proc_macro2::Span::call_site(), "Invalid inert attr"));
                            }

                            if let Lit::Str(s) = &nm.lit {
                                arg_ident = Some(Ident::new(&s.value(), proc_macro2::Span::call_site()));
                                break;
                            }
                        }
                    }

                }

            } // for attr


            let mut add_default = true;

            if let Some(arg_ident) = arg_ident {

                if let syn::Type::Path(ty_path) = ty {
                    let path_seg = ty_path.path.segments.first().unwrap();

                    if let syn::PathArguments::AngleBracketed(path_args) = &path_seg.arguments {
                        let arg = path_args.args.first().unwrap();

                        if let syn::GenericArgument::Type(arg_ty) = arg {

                            q.extend(
                                quote::quote! {
                                    pub fn #arg_ident(&mut self, #arg_ident: #arg_ty) -> &mut Self {
                                        if let None = self.#name {
                                            self.#name = Some(std::vec::Vec::new());
                                        }
                                        self.#name.as_mut().unwrap().push(#arg_ident);
                                        self
                                    }
                                }
                            );

                            add_default = arg_ident != *name;


                        }

                    }
                }

            }

            if add_default {
                q.extend(
                    quote::quote! {
                        pub fn #name (&mut self, #name: #ty) -> &mut Self {
                            self.#name = Some(#name);
                            self
                        }
                    }
                );
            }
        };

        Ok(q)
}

/// Create TokenStream for build method
fn create_build_ts(input_ident: &proc_macro2::Ident, named_fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    let recurse = named_fields.named.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();

        use syn::spanned::Spanned;

        if is_field_optional(f).is_some() {
            // just clone the optional value
            quote::quote_spanned!(f.span() =>
                #name: self.#name.to_owned(),
            )
        } else {
            // return error if field is none, otherwise clone
            quote::quote_spanned!(f.span() =>
                #name: self.#name.as_ref()
                                 .ok_or::<std::boxed::Box<dyn std::error::Error>>(std::string::String::from("ERROR").into())?
                                 .to_owned(),
            )
        }
    });

    quote::quote! {

        pub fn build(&mut self) -> std::result::Result<#input_ident, std::boxed::Box<dyn std::error::Error>> {
            Ok(#input_ident {
                #( #recurse )*
            })
        }

    }
}

/// Checks weather a given field is optional
fn is_field_optional(field: &syn::Field) -> Option<&syn::Type> {

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
