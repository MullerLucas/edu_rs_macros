
#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let input_ident = &input.ident;
    // let ident = Ident::new(&format!("{}Builder", input.ident), Span::call_site());
    let builder_ident = quote::format_ident!("{}Builder", &input.ident);
    let setters = create_setters(&input.data);



    let expanded = quote::quote! {
        impl #input_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }

        pub struct #builder_ident {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl #builder_ident {
            #setters
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn create_setters(data: &syn::Data) -> proc_macro2::TokenStream {

    match *data {
        syn::Data::Struct(ref d) => {
            match d.fields {
                syn::Fields::Named(ref fields) => {

                    let mut q = quote::quote!();
                    fields.named.iter().for_each(|f| {
                        let name = f.ident.as_ref().unwrap();
                        let ty = &f.ty;

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
