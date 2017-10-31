// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Matchable)]
pub fn matchable(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_matchable(&ast);
    gen.parse().unwrap()
}

fn impl_matchable(ast: &syn::DeriveInput) -> quote::Tokens {

    let path = &quote! {
        ::runtime_pattern::
    };

    // Ideally I'd do this on a per-type basis based on which traits the type in
    // question derived, but `syn::DeriveInput` doesn't give `derive`
    // attributes.
    let derives = quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
    };

    let visibility =  if ast.vis == syn::Visibility::Public {
        quote! { pub }
    } else {
        quote! {}
    };

    let name = &ast.ident;
    let ty_params: &Vec<syn::Ident> = &ast.generics.ty_params.iter().map(|ty_param|
        ty_param.ident.clone()
    ).collect();

    let ty_params_tokens = if ty_params.len() == 0 {
        quote! {}
    } else {
        quote! {
            < #( #ty_params, )* >
        }
    };

    let ty_params_pattern: &Vec<syn::Ident> = &ty_params.iter().map(|ty_param| {
        let s = format!("{}Pattern", ty_param);
        syn::Ident::from(s)
    }).collect();

    let generics_list_struct = if ty_params.len() == 0 {
        quote! {}
    } else {
        quote! {
            < #( #ty_params, )* #( #ty_params_pattern, )* >
        }
    };

    let path_where_clause = &vec![path; ty_params.len()];

    let impl_where_clause = if ty_params.len() == 0 {
        quote! {}
    } else {
        quote! {
            where
                #(
                    #ty_params: #path_where_clause Matchable<Pattern = #ty_params_pattern>,
                )*
        }
    };

    let pattern_name = &syn::Ident::new(format!("{}Pattern", name));

    match &ast.body {
        &syn::Body::Enum(ref variants) => {
            let mut pattern_enum_variants: Vec<quote::Tokens> = Vec::new();
            let mut match_branches: Vec<quote::Tokens> = Vec::new();

            for variant in variants.iter() {
                let variant_ident = &variant.ident;
                match variant.data {
                    syn::VariantData::Unit => {
                        pattern_enum_variants.push(
                            quote!{
                                #variant_ident ,
                            },
                        );
                        match_branches.push(
                            quote!{
                                (&#name::#variant_ident, &mut #pattern_name::#variant_ident) => true,
                            }
                        )
                    }

                    syn::VariantData::Tuple(ref fields) => {
                        let field_names = &create_names(fields.len(), "field_");
                        let field_pattern_names = &create_names(fields.len(), "pattern_");
                        let field_types = &extract_field_types(fields);
                        let field_types2 = &extract_field_types2(field_types, ty_params, &path);
                        let paths = &vec![path; fields.len()];

                        pattern_enum_variants.push(
                            quote!{
                                #variant_ident (
                                    #(
                                        #paths WholePattern<
                                            #field_types,
                                            #field_types2,
                                        >,
                                    )*
                                ),
                            }
                        );

                        match_branches.push(
                            quote!{
                                (
                                    &#name::#variant_ident(
                                        #(
                                            ref #field_names,
                                        )*
                                    ),
                                    &mut #pattern_name::#variant_ident(
                                        #(
                                            ref mut #field_pattern_names,
                                        )*
                                    )
                                ) => {
                                    #(
                                        if ! #paths Matchable::matc(#field_names, #field_pattern_names) {
                                            return false;
                                        }
                                    )*
                                    return true;
                                }
                            }
                        )
                    }

                    syn::VariantData::Struct(ref fields) => {
                        let field_names = &extract_field_names(fields);
                        let field_types = &extract_field_types(fields);
                        let field_types2 = &extract_field_types2(field_types, ty_params, path);
                        let paths = &vec![path; fields.len()];

                        let append = |s| {
                            field_names.iter().map(|name| {
                                let new_name = format!("{}{}", name.to_string(), s);
                                syn::Ident::new(new_name.as_ref())
                            }).collect()
                        };

                        let field_names_appended1: &Vec<syn::Ident> = &append("_1");
                        let field_names_appended2: &Vec<syn::Ident> = &append("_2");

                        pattern_enum_variants.push(
                            quote!{
                                #variant_ident
                                {
                                    #(
                                        #field_names: #paths WholePattern<
                                            #field_types,
                                            #field_types2,
                                        >,
                                    )*
                                },
                            }
                        );

                        match_branches.push(
                            quote! {
                                (
                                    &#name::#variant_ident {
                                        #(
                                            #field_names: ref #field_names_appended1,
                                        )*
                                    },
                                    &mut #pattern_name::#variant_ident {
                                        #(
                                            #field_names: ref mut #field_names_appended2,
                                        )*
                                    }
                                ) => {
                                    #(
                                        if ! #paths Matchable::matc(#field_names_appended1, #field_names_appended2) {
                                            return false;
                                        }
                                    )*
                                    return true;
                                }
                            }
                        );
                    }
                }
            }

            let result =

            quote! {
                #derives
                #visibility
                enum #pattern_name #generics_list_struct {
                    #(
                        #pattern_enum_variants
                    )*
                }

                impl #generics_list_struct #path Matchable for #name #ty_params_tokens
                #impl_where_clause
                {
                    type Pattern = #pattern_name #generics_list_struct;

                    fn match_impl(&self, pattern: &mut Self::Pattern) -> bool {
                        match (self, pattern) {
                            #(
                                #match_branches
                            )*
                            _ => false,
                        }
                    }
                }
            }

            ;
            if name == &syn::Ident::from("XFullMnemonic") {
                panic!("{:?}", result);
            } else {
                result
            }

        },

        &syn::Body::Struct(syn::VariantData::Unit) => {
            // Unit struct: no need for a separate Pattern type, and everything
            // matches
            quote! {
                impl #path Matchable for #name {
                    type Pattern = Self;

                    fn match_impl(&self, _pattern: &mut Self::Pattern) -> bool {
                        true
                    }
                }
            }
        },

        &syn::Body::Struct(syn::VariantData::Tuple(ref fields)) => {
            let field_types: &Vec<syn::Ty> = &fields.iter().map(|f| f.ty.clone()).collect();

            // Given type parameters <T, ...>, the Pattern struct takes
            // additional type parameters of the form TPattern, ..., and these
            // need to be passed directly to WholePattern, rather than passing
            // T::Pattern. This way we we can use `derive` and the conditional
            // implementation will actually work correctly.
            let field_types2 = extract_field_types2(field_types, ty_params, path);

            let field_names = &create_names(fields.len(), "");
            let field_names2 = field_names;

            let paths = &vec![path; field_types.len()];

            quote! {
                #derives
                #visibility
                struct #pattern_name #generics_list_struct (
                    #( 
                        pub
                        #paths WholePattern < 
                            #field_types,
                            #field_types2,
                        >,
                    )*
                );

                impl #generics_list_struct #path Matchable for #name #ty_params_tokens
                #impl_where_clause
                {
                    type Pattern = #pattern_name #generics_list_struct;

                    fn match_impl(&self, pattern: &mut Self::Pattern) -> bool {
                        #(
                            if ! #paths Matchable::matc(&self.#field_names, &mut pattern.#field_names2) {
                                return false;
                            }
                        )*
                        return true;
                    }
                }
            }
        },

        &syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
            let field_types: &Vec<syn::Ty> = &fields.iter().map(|f| f.ty.clone()).collect();
            let field_types2 = &extract_field_types2(field_types, ty_params, path);
            let field_names = &extract_field_names(fields);
            let field_names2 = field_names;
            let paths = &vec![path; field_types.len()];

            quote! {
                #derives
                #visibility
                struct #pattern_name #generics_list_struct {
                    #( 
                        pub
                        #field_names:
                        #paths WholePattern < 
                            #field_types,
                            #field_types2,
                        >,
                    )*
                }

                impl #generics_list_struct #path Matchable for #name #ty_params_tokens
                #impl_where_clause
                {
                    type Pattern = #pattern_name #generics_list_struct;

                    fn match_impl(&self, pattern: &mut Self::Pattern) -> bool {
                        #(
                            if ! #paths Matchable::matc(&self.#field_names, &mut pattern.#field_names2) {
                                return false;
                            }
                        )*
                        return true;
                    }
                }
            }
        }
    }
}

fn extract_field_names(fields: &Vec<syn::Field>) -> Vec<syn::Ident> {
    fields.iter().map(|field| {
        match field.ident {
            Some(ref id) => id.clone(),
            None => panic!("Field without an ident"),
        }
    }).collect()
}

fn extract_field_types(fields: &Vec<syn::Field>) -> Vec<syn::Ty> {
    fields.iter().map(|field| field.ty.clone()).collect()
}

fn extract_field_types2(
    field_types: &Vec<syn::Ty>,
    ty_params: &Vec<syn::Ident>,
    path: &quote::Tokens,
) -> Vec<quote::Tokens> {
    field_types.iter().map(|ty| {
        if let &syn::Ty::Path(_, syn::Path { ref segments, .. }) = ty {
            if let Some(x) = segments.last()  {
                let x_id = &x.ident;
                let x_pattern = syn::Ident::from(format!("{}Pattern", x_id));
                if ty_params.contains(&x.ident) {
                    return quote! {
                        #x_pattern
                    }
                }
            }
        }
        return quote! {
            <#ty as #path Matchable>::Pattern
        }
    }).collect()
}

fn create_names(count: usize, prefix: &str) -> Vec<syn::Ident> {
    (0 .. count).map(|i| {
        let s = format!("{}{:0>9X}", prefix, i);
        syn::Ident::new(s)
    }).collect()
}
