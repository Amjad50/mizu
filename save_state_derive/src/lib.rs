mod attrs;
mod parsing;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Result};

use parsing::{Container, ContainerData, Fields, FieldsType, Variant};

#[proc_macro_derive(Savable, attributes(savable))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> Result<TokenStream2> {
    let container = Container::new(&input)?;

    if container.attrs.use_serde {
        impl_for_serde_full(&container)
    } else {
        impl_for_savables(&container)
    }
}

fn impl_for_serde_full(container: &Container) -> Result<TokenStream2> {
    let ident = &container.ident;
    let (impl_generics, ty_generics, where_clause) = container.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::save_state::Savable for #ident #ty_generics #where_clause {
            #[inline]
            fn save<W: ::std::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::save_state::Result<()> {
                ::save_state::serialize_into(writer, self)?;
                ::std::result::Result::Ok(())
            }

            #[inline]
            fn load<R: ::std::io::Read>(
                &mut self,
                reader: &mut R,
            ) -> ::save_state::Result<()> {
                let obj = ::save_state::deserialize_from(reader)?;

                let _ = ::std::mem::replace(self, obj);
                ::std::result::Result::Ok(())
            }
        }
    })
}

fn impl_fields_for_save(fields: &Fields, ident_prefix: TokenStream2) -> Vec<TokenStream2> {
    let idents = fields.unskipped_idents();

    fields
        .unskipped_fields
        .iter()
        .zip(idents)
        .map(|(f, ident)| {
            if f.attrs.use_serde {
                quote!(::save_state::serialize_into(&mut writer, #ident_prefix #ident)?;)
            } else {
                quote!(::save_state::Savable::save(#ident_prefix #ident, &mut writer)?;)
            }
        })
        .collect()
}

fn impl_fields_for_load(fields: &Fields, ident_prefix: TokenStream2) -> Vec<TokenStream2> {
    let idents = fields.unskipped_idents();

    fields.unskipped_fields
        .iter()
        .zip(idents)
        .map(|(f, ident)| {
            if f.attrs.use_serde {
                quote!(let _ = ::std::mem::replace(#ident_prefix #ident, ::save_state::deserialize_from(&mut reader)?);)
            } else {
                quote!(::save_state::Savable::load(#ident_prefix #ident, &mut reader)?;)
            }
        })
        .collect()
}

fn get_fields_impl_size_sum(fields: &Fields, ident_prefix: TokenStream2) -> TokenStream2 {
    let idents = fields.unskipped_idents();

    let all_fields = fields
        .unskipped_fields
        .iter()
        .zip(idents.iter())
        .map(|(f, ident)| {
            if f.attrs.use_serde {
                quote!(::save_state::serialized_size(#ident_prefix #ident)?)
            } else {
                quote!(::save_state::Savable::save_size(#ident_prefix #ident)?)
            }
        });

    if idents.is_empty() {
        quote!(0)
    } else {
        quote!(#(#all_fields)+*)
    }
}

fn impl_for_savables(container: &Container) -> Result<TokenStream2> {
    match &container.data {
        ContainerData::Struct(fields) => impl_for_struct(container, fields),
        ContainerData::Enum(variants) => impl_for_enum(container, variants),
    }
}

fn impl_for_struct(container: &Container, fields: &Fields) -> Result<TokenStream2> {
    let ident = &container.ident;

    let save_fields = impl_fields_for_save(fields, quote!(&self.));
    let load_fields = impl_fields_for_load(fields, quote!(&mut self.));
    let size_sum = get_fields_impl_size_sum(fields, quote!(&self.));
    let (impl_generics, ty_generics, where_clause) = container.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::save_state::Savable for #ident #ty_generics #where_clause {
            #[inline]
            fn save<W: ::std::io::Write>(
                &self,
                mut writer: &mut W,
            ) -> ::save_state::Result<()> {
                #(#save_fields)*
                ::std::result::Result::Ok(())
            }

            #[inline]
            fn load<R: ::std::io::Read>(
                &mut self,
                mut reader: &mut R,
            ) -> ::save_state::Result<()> {
                #(#load_fields)*
                ::std::result::Result::Ok(())
            }

            #[inline]
            fn save_size(&self) -> ::save_state::Result<u64> {
                ::std::result::Result::Ok(#size_sum)
            }
        }
    })
}

fn impl_for_enum(container: &Container, variants: &[Variant]) -> Result<TokenStream2> {
    let ident = &container.ident;

    let save_variants = variants.iter().map(|v| {
        let ident = v.ident.clone();
        let position = v.position;
        let all_fields = v.fields.all_idents();

        // only save fields that are not skipped
        let save_fields = impl_fields_for_save(&v.fields, quote!());

        let fields_part = match v.fields.fields_type {
            FieldsType::Named => {
                quote!({#(#all_fields),*})
            }
            FieldsType::Unnamed => {
                quote!((#(#all_fields),*))
            }
            FieldsType::Unit => {
                quote!()
            }
        };

        quote!(Self::#ident #fields_part => {
            <usize as ::save_state::Savable>::save(&#position, &mut writer)?;
            #(#save_fields)*
        })
    });

    let load_variants = variants.iter().map(|v| {
        let ident = v.ident.clone();
        let position = v.position;
        let all_fields = v.fields.all_idents();

        // perform intialiaztion with `Default` to all fields
        let default_initializations = all_fields
            .iter()
            .map(|ident| quote!(let mut #ident = ::std::default::Default::default();));

        // only perform load for the unskipped fields
        let load_fields = impl_fields_for_load(&v.fields, quote!(&mut));

        let result = match v.fields.fields_type {
            FieldsType::Named => {
                quote!(Self::#ident{#(#all_fields),*})
            }
            FieldsType::Unnamed => {
                quote!(Self::#ident(#(#all_fields),*))
            }
            FieldsType::Unit => {
                quote!(Self::#ident)
            }
        };

        quote!(#position => {
            #(#default_initializations)*
            #(#load_fields)*
            #result
        })
    });

    let (impl_generics, ty_generics, where_clause) = container.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::save_state::Savable for #ident #ty_generics #where_clause {
            #[inline]
            fn save<W: ::std::io::Write>(
                &self,
                mut writer: &mut W,
            ) -> ::save_state::Result<()> {
                match self {
                    #(#save_variants)*
                }
                ::std::result::Result::Ok(())
            }

            #[inline]
            fn load<R: ::std::io::Read>(
                &mut self,
                mut reader: &mut R,
            ) -> ::save_state::Result<()> {
                let mut position: usize = 0;
                position.load(&mut reader)?;
                let enum_value = match position {
                    #(#load_variants)*
                    v => return Err(::save_state::Error::InvalidEnumVariant(v))
                };

                *self = enum_value;

                ::std::result::Result::Ok(())
            }
        }
    })
}
