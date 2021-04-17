use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, Data, DataStruct, DeriveInput, Fields, Generics, Ident,
    Index, Result, Token, Type, TypeParamBound, WherePredicate,
};

use crate::attrs::{ContainerAttrs, FieldAttrs};

pub struct Field {
    pub attrs: FieldAttrs,
    pub ident: Option<Ident>,
    pub position: Index,
    pub ty: Type,
}

impl Field {
    pub fn new(field: &syn::Field, position: usize) -> Result<Self> {
        let attrs = FieldAttrs::new(field)?;

        Ok(Self {
            attrs,
            ident: field.ident.clone(),
            position: Index::from(position),
            ty: field.ty.clone(),
        })
    }

    pub fn ident_tokens(&self) -> TokenStream2 {
        let position = &self.position;

        self.ident
            .as_ref()
            .map_or_else(|| quote!(#position), |ident| quote!(#ident))
    }
}

pub struct Container {
    pub ident: Ident,
    pub attrs: ContainerAttrs,
    pub fields: Vec<Field>,
    pub generics: Generics,
}

impl Container {
    pub fn new(input: &DeriveInput) -> Result<Self> {
        let attrs = ContainerAttrs::new(input)?;

        let fields = match &input.data {
            Data::Struct(DataStruct { fields, .. }) => match fields {
                Fields::Named(named) => Self::parse_fields(named.named.iter())?,
                Fields::Unnamed(unnamed) => Self::parse_fields(unnamed.unnamed.iter())?,
                Fields::Unit => Vec::new(),
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "Savable is only implementable for structs",
                ));
            }
        };

        let generics = Self::build_generics(&input.generics, &attrs, &fields);

        Ok(Self {
            ident: input.ident.clone(),
            generics,
            attrs,
            fields,
        })
    }

    fn parse_fields<'a, I: Iterator<Item = &'a syn::Field>>(fields: I) -> Result<Vec<Field>> {
        let all_fields = fields
            .enumerate()
            .map(|(i, f)| Field::new(f, i))
            .collect::<Result<Vec<_>>>()?;

        // remove all skipped fields
        Ok(all_fields.into_iter().filter(|f| !f.attrs.skip).collect())
    }

    fn build_generics(generics: &Generics, attrs: &ContainerAttrs, _fields: &[Field]) -> Generics {
        // TODO: make sure to only add predicates for used generics
        let bounds: Punctuated<TypeParamBound, Token![+]> = if attrs.use_serde {
            parse_quote!(::serde::Serialize + ::serde::de::DeserializeOwned)
        } else {
            parse_quote!(::save_state::Savable)
        };

        let new_predicates = generics
            .type_params()
            .map(|param| param.ident.clone())
            .map::<WherePredicate, _>(|ident| parse_quote!(#ident: #bounds))
            .collect::<Vec<_>>();

        let mut new_generics = generics.clone();
        new_generics
            .make_where_clause()
            .predicates
            .extend(new_predicates);

        new_generics
    }
}
