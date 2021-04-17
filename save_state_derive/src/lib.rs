mod attrs;
mod parsing;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Result};

use parsing::{Container, Field};

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
            ) -> Result<(), ::save_state::SaveError> {
                ::save_state::bincode::serialize_into(writer, self)?;
                Ok(())
            }

            #[inline]
            fn load<R: ::std::io::Read>(
                &mut self,
                reader: &mut R,
            ) -> Result<(), ::save_state::SaveError> {
                let obj = ::save_state::bincode::deserialize_from(reader)?;

                let _ = ::std::mem::replace(self, obj);
                Ok(())
            }
        }
    })
}

fn impl_fields_for_save(fields: &[Field]) -> Vec<TokenStream2> {
    fields
        .iter()
        .map(|f| {
            let ident = f.ident_tokens();

            if f.attrs.use_serde {
                quote!(::save_state::bincode::serialize_into(&mut writer, &self.#ident)?;)
            } else {
                quote!(::save_state::Savable::save(&self.#ident, &mut writer)?;)
            }
        })
        .collect()
}

fn impl_fields_for_load(fields: &[Field]) -> Vec<TokenStream2> {
    fields
        .iter()
        .map(|f| {
            let ident = f.ident_tokens();

            if f.attrs.use_serde {
                quote!(let _ = ::std::mem::replace(&mut self.#ident, ::save_state::bincode::deserialize_from(&mut reader)?);)
            } else {
                quote!(::save_state::Savable::load(&mut self.#ident, &mut reader)?;)
            }
        })
        .collect()
}

fn get_fields_impl_size_sum(fields: &[Field]) -> TokenStream2 {
    let all_fields = fields.iter().map(|f| {
        let ident = f.ident_tokens();

        if f.attrs.use_serde {
            quote!(::save_state::bincode::serialized_size(&self.#ident).map_err::<::save_state::SaveError, _>(|e| e.into())?)
        } else {
            quote!(::save_state::Savable::save_size(&self.#ident)?)
        }
    });

    if fields.is_empty() {
        quote!(0)
    } else {
        quote!(#(#all_fields)+*)
    }
}

fn impl_for_savables(container: &Container) -> Result<TokenStream2> {
    let ident = &container.ident;

    let save_fields = impl_fields_for_save(&container.fields);
    let load_fields = impl_fields_for_load(&container.fields);
    let size_sum = get_fields_impl_size_sum(&container.fields);
    let (impl_generics, ty_generics, where_clause) = container.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::save_state::Savable for #ident #ty_generics #where_clause {
            #[inline]
            fn save<W: ::std::io::Write>(
                &self,
                mut writer: &mut W,
            ) -> Result<(), ::save_state::SaveError> {
                #(#save_fields)*
                Ok(())
            }

            #[inline]
            fn load<R: ::std::io::Read>(
                &mut self,
                mut reader: &mut R,
            ) -> Result<(), ::save_state::SaveError> {
                #(#load_fields)*
                Ok(())
            }

            #[inline]
            fn save_size(&self) -> Result<u64, ::save_state::SaveError> {
                Ok(#size_sum)
            }
        }
    })
}
