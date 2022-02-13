#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let input_ident = &input.ident;
    let builder_ident = quote::format_ident!("{}Builder", &input.ident);

    // extract relevant informations
    // -----------------------------
    let raw_fields = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(n),
            ..
        }) => n,
        _ => unimplemented!(),
    };

    let fields = raw_fields.named.iter().filter_map(|f| {
        f.ident.as_ref().map(|ident|
                             (ident, &f.ty, inner_ty(&f.ty, "Option"), inner_for_attr(f)))
    });

    // construct token-streams
    // -----------------------
    let ts_builder_def = fields
        .clone()
        .map(|(name, ty, option, attr)| match (option, attr) {
            (None, None) => quote::quote! { #name: std::option::Option<#ty> },
            (Some(ty), None) => quote::quote! { #name: std::option::Option<#ty> },
            (None, Some((_, ty))) => quote::quote! { #name: std::vec::Vec<#ty> },
            _ => unimplemented!(),
        });

    let ts_builder_init = fields.clone().map(|(name, _, _, attr)| match attr {
        None => quote::quote! { #name: None },
        Some(_) => quote::quote! { #name: Vec::new() },
    });

    let ts_build_extract = fields
        .clone()
        .map(|(name, _, option, attr)| match (option, attr) {
            (None, None) => quote::quote! { #name: self.#name.clone()? },
            (Some(_), None) => quote::quote! { #name: self.#name.clone() },
            (None, Some(_)) => quote::quote! { #name: self.#name.clone() },
            _ => unimplemented!(),
        });

    let mut ts_builder_setters = quote::quote! {};

    fields
        .clone()
        .for_each(|(name, ty, option, attr)| match (option, attr) {
            (None, None) => ts_builder_setters.extend(quote::quote! {
                pub fn #name(&mut self, value: #ty) -> &mut Self {
                    self.#name = Some(value);
                    self
                }
            }),

            (Some(ty), None) => ts_builder_setters.extend(quote::quote! {
                pub fn #name(&mut self, value: #ty) -> &mut Self {
                    self.#name = Some(value);
                    self
                }
            }),

            (None, Some((arg_name, arg_ty))) => {
                ts_builder_setters.extend(quote::quote! {
                    pub fn #arg_name(&mut self, value: #arg_ty) -> &mut Self {
                        self.#name.push(value);
                        self
                    }
                });

                if *name != arg_name {
                    ts_builder_setters.extend(quote::quote! {
                        pub fn #name(&mut self, value: #ty) -> &mut Self {
                            self.#name = value;
                            self
                        }
                    });
                }
            }

            _ => unimplemented!(),
        });

    // combine all token-streams
    // -------------------------
    let ts_expanded = quote::quote! {
        impl #input_ident {
            fn builder() -> #builder_ident {
                #builder_ident {
                    #(
                        #ts_builder_init,
                    )*
                }
            }
        }

        #[derive(Debug)]
        struct #builder_ident {
            #(
                #ts_builder_def,
            )*
        }

        impl #builder_ident {
            pub fn build(&self) -> std::option::Option<#input_ident> {
                Some(#input_ident {
                    #(
                        #ts_build_extract,
                    )*
                })
            }

            #ts_builder_setters
        }
    };

    proc_macro::TokenStream::from(ts_expanded)
}

fn inner_ty<'a>(ty: &'a syn::Type, outer_ty: &str) -> Option<&'a syn::Type> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { segments, .. },
        ..
    }) = ty
    {
        let segment = &segments[0];

        if segment.ident == outer_ty {
            if let syn::PathArguments::AngleBracketed(generic) = &segment.arguments {
                if let syn::GenericArgument::Type(ty) = generic.args.first().unwrap() {
                    return Some(ty);
                }
            }
        }
    }

    None
}

fn inner_for_attr(field: &syn::Field) -> Option<(syn::Ident, &syn::Type)> {
    if let Some(attr) = field.attrs.first() {
        if let Ok(syn::Meta::List(meta)) = attr.parse_meta() {
            if let Some(seg) = meta.path.segments.first() {
                if seg.ident == "builder" {
                    if let Some(syn::NestedMeta::Meta(syn::Meta::NameValue(nested))) =
                        meta.nested.first()
                    {
                        if let Some(nested_seg) = nested.path.segments.first() {
                            if nested_seg.ident == "each" {
                                if let syn::Lit::Str(lit) = &nested.lit {
                                    return Some((
                                        syn::Ident::new(
                                            &lit.value(),
                                            proc_macro2::Span::call_site(),
                                        ),
                                        inner_ty(&field.ty, "Vec").unwrap(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
