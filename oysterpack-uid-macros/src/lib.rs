/*
 * Copyright 2018 OysterPack Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

//! Provides macros for its [oysterpack_uid](https://crates.io/crates/oysterpack_uid) sister crate.
//! This crate is exported by [oysterpack_uid](https://crates.io/crates/oysterpack_uid).

#![doc(html_root_url = "https://docs.rs/oysterpack_uid_macros/0.1.0")]
#![recursion_limit = "128"]

extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, AttributeArgs};

// TODO: add support for `#[ulid(domain = "DOMAIN_NAME")]` - provides an Into<DomainULID> impl

/// ulid attribute macro - augments u128 tuple structs to be ULID compatible. It provides the following:
/// - conversion to and from ULID
/// - conversion from a DomainULID
/// - fmt::Display implementation mirrors ULID
/// - `fn ulid() -> ULID` getter
/// - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]`
/// ## For Example
/// <pre>
/// use oysterpack_uid::macros::ulid;
///
/// #[ulid]
/// pub struct UserId(pub u128);
/// </pre>
///
/// ### Produces the following code
/// <pre>
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
/// pub struct UserId(pub u128);
///
/// impl From<UserId> for oysterpack_uid::ULID {
///     fn from(ulid: UserId) -> oysterpack_uid::ULID {
///         ulid.0.into()
///     }
/// }
///
/// impl From<oysterpack_uid::ULID> for UserId{
///     fn from(ulid: oysterpack_uid::ULID) -> UserId{
///         UserId(ulid.into())
///     }
/// }
///
/// impl From<oysterpack_uid::DomainULID> for UserId{
///     fn from(ulid: oysterpack_uid::DomainULID) -> UserId{
///         UserId(ulid.ulid().into())
///     }
/// }
///
/// impl UserId{
///     /// returns the ID as a ULID
///     pub fn ulid(&self) -> oysterpack_uid::ULID {
///         self.0.into()
///     }
/// }
///
/// impl std::fmt::Display for UserId{
///     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
///         let ulid: oysterpack_uid::ULID = self.0.into();
///         f.write_str(ulid.to_string().as_str())
///     }
/// }
/// </pre>
#[proc_macro_attribute]
pub fn ulid(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item: syn::Item = syn::parse(item).unwrap();

    match item {
        syn::Item::Struct(ref item_struct) if is_u128_tuple_struct(item_struct) => {
            let struct_ident = &item_struct.ident;
            let output = quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
                #item_struct

                impl From<#struct_ident> for oysterpack_uid::ULID {
                    fn from(ulid: #struct_ident) -> oysterpack_uid::ULID {
                        ulid.0.into()
                    }
                }

                impl From<oysterpack_uid::ULID> for #struct_ident {
                    fn from(ulid: oysterpack_uid::ULID) -> #struct_ident {
                        #struct_ident(ulid.into())
                    }
                }

                impl From<oysterpack_uid::DomainULID> for #struct_ident {
                    fn from(ulid: oysterpack_uid::DomainULID) -> #struct_ident {
                        #struct_ident(ulid.ulid().into())
                    }
                }

                impl #struct_ident {
                    /// returns the ID as a ULID
                    pub fn ulid(&self) -> oysterpack_uid::ULID {
                        self.0.into()
                    }

                    /// generates a new unique identifier based on a ULID
                    pub fn generate() -> #struct_ident {
                        #struct_ident(oysterpack_uid::ULID::generate().into())
                    }
                }

                impl std::fmt::Display for #struct_ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        let ulid: oysterpack_uid::ULID = self.0.into();
                        f.write_str(ulid.to_string().as_str())
                    }
                }
            };
            output.into()
        }
        syn::Item::Struct(ref item_struct) if is_ulid_tuple_struct(item_struct) => {
            let struct_ident = &item_struct.ident;
            let output = quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
                #item_struct

                impl From<#struct_ident> for oysterpack_uid::ULID {
                    fn from(ulid: #struct_ident) -> oysterpack_uid::ULID {
                        ulid.0
                    }
                }

                impl From<oysterpack_uid::ULID> for #struct_ident {
                    fn from(ulid: oysterpack_uid::ULID) -> #struct_ident {
                        #struct_ident(ulid)
                    }
                }

                impl From<oysterpack_uid::DomainULID> for #struct_ident {
                    fn from(ulid: oysterpack_uid::DomainULID) -> #struct_ident {
                        #struct_ident(ulid.ulid())
                    }
                }

                impl #struct_ident {
                    /// returns the ID as a ULID
                    pub fn ulid(&self) -> oysterpack_uid::ULID {
                        self.0
                    }

                    /// generates a new unique identifier based on a ULID
                    pub fn generate() -> #struct_ident {
                        #struct_ident(oysterpack_uid::ULID::generate())
                    }

                    /// Returns a new ULID with the random part incremented by one.
                    /// Overflowing the random part generates a new ULID, i.e., with a new timestamp portion.
                    pub fn increment(self) -> #struct_ident {
                        let prev = self.0;
                        let ulid = self.0.increment();
                        if ulid < prev {
                            Self::generate()
                        } else {
                            #struct_ident(ulid)
                        }
                    }
                }

                impl std::fmt::Display for #struct_ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        f.write_str(self.0.to_string().as_str())
                    }
                }
            };
            output.into()
        }
        _ => {
            // TODO: replace with Diagnostic error when it becomes stable on proc_macro
            panic!("#[ulid] is only supported on u128 or oysterpack_uid::ULID tuple structs")
        }
    }
}

#[proc_macro_attribute]
pub fn domain(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let item: syn::Item = syn::parse(item).unwrap();

    let domain_name = domain_name(args);

    match item {
        syn::Item::Struct(ref item_struct) => {
            let struct_ident = &item_struct.ident;
            let output = quote! {
                #item_struct

                impl oysterpack_uid::HasDomain for #struct_ident {
                    const DOMAIN: oysterpack_uid::Domain = oysterpack_uid::Domain(stringify!(#domain_name));
                }
            };
            output.into()
        }
        _ => panic!("#[domain(name)] is only supported on structs"),
    }
}

fn domain_name(args: syn::AttributeArgs) -> syn::Ident {
    if args.len() != 1 {
        panic!("domain name was not specified: #[domain(name)] ");
    }

    match args.first().unwrap() {
        syn::NestedMeta::Meta(meta) => match meta {
            syn::Meta::Word(ident) => ident.clone(),
            _ => panic!("domain attribute format is #[domain(name)]"),
        },
        _ => panic!("domain attribute format is #[domain(name)]"),
    }
}

fn is_u128_tuple_struct(item_struct: &syn::ItemStruct) -> bool {
    match &item_struct.fields {
        syn::Fields::Unnamed(fields) => match fields.unnamed.len() {
            1 => {
                let field = *fields.unnamed.first().unwrap().value();
                field_matches_type(field, "u128")
            }
            _ => false,
        },
        _ => false,
    }
}

fn is_ulid_tuple_struct(item_struct: &syn::ItemStruct) -> bool {
    match &item_struct.fields {
        syn::Fields::Unnamed(fields) => match fields.unnamed.len() {
            1 => {
                let field = *fields.unnamed.first().unwrap().value();
                field_matches_type(field, "ULID")
                    || field_matches_type(field, "oysterpack_uid::ULID")
                    || field_matches_type(field, "uid::ULID")
            }
            _ => false,
        },
        _ => false,
    }
}

fn field_matches_type(field: &syn::Field, ty: &str) -> bool {
    match &field.ty {
        syn::Type::Path(type_path) => path_to_string(type_path) == ty,
        _ => false,
    }
}

fn path_to_string(path: &syn::TypePath) -> String {
    path.path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .fold("".to_string(), |acc, s| {
            if acc == "" {
                s
            } else {
                format!("{}::{}", acc, s)
            }
        })
}
