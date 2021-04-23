use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_quote, punctuated::Punctuated, Data, DataEnum, DataStruct, DeriveInput, Generics, Ident,
    Index, Result, Token, Type, TypeParamBound, WherePredicate,
};

use crate::attrs::{ContainerAttrs, FieldAttrs};

pub enum FieldsType {
    Named,
    Unnamed,
    Unit,
}

pub enum FieldsLocation {
    Struct,
    EnumVariant,
}

pub struct Fields {
    pub all_fields: Vec<Field>,
    pub unskipped_fields: Vec<Field>,
    pub fields_type: FieldsType,
    pub location: FieldsLocation,
}

impl Fields {
    pub fn new(fields: &syn::Fields, location: FieldsLocation) -> Result<Self> {
        let (fields, fields_type) = match &fields {
            syn::Fields::Named(named) => (named.named.clone(), FieldsType::Named),
            syn::Fields::Unnamed(unnamed) => (unnamed.unnamed.clone(), FieldsType::Unnamed),
            syn::Fields::Unit => (Default::default(), FieldsType::Unit),
        };

        let all_fields = fields
            .iter()
            .enumerate()
            .map(|(i, f)| Field::new(f, i))
            .collect::<Result<Vec<_>>>()?;

        // remove all skipped fields
        let unskipped_fields = all_fields
            .iter()
            .cloned()
            .filter(|f| !f.attrs.skip)
            .collect();

        Ok(Self {
            all_fields,
            unskipped_fields,
            fields_type,
            location,
        })
    }

    pub fn unskipped_idents(&self) -> Vec<TokenStream2> {
        self.formatted_idents(&self.unskipped_fields)
    }

    pub fn all_idents(&self) -> Vec<TokenStream2> {
        self.formatted_idents(&self.all_fields)
    }

    fn formatted_idents(&self, fields: &[Field]) -> Vec<TokenStream2> {
        let pos_ident_handler = match self.location {
            FieldsLocation::Struct => |position| quote!(#position),
            FieldsLocation::EnumVariant => {
                |position| format_ident!("_{}", position).to_token_stream()
            }
        };

        fields
            .iter()
            .map(|f| {
                f.ident
                    .as_ref()
                    .map_or_else(|| pos_ident_handler(&f.position), |ident| quote!(#ident))
            })
            .collect()
    }
}

#[derive(Clone)]
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
}

pub struct Variant {
    pub fields: Fields,
    pub ident: Ident,
    pub position: usize,
}

impl Variant {
    fn new(variant: &syn::Variant, position: usize) -> Result<Self> {
        Ok(Self {
            ident: variant.ident.clone(),
            position,
            fields: Fields::new(&variant.fields, FieldsLocation::EnumVariant)?,
        })
    }
}

pub enum ContainerData {
    Struct(Fields),
    Enum(Vec<Variant>),
}

pub struct Container {
    pub ident: Ident,
    pub attrs: ContainerAttrs,
    pub data: ContainerData,
    pub generics: Generics,
}

impl Container {
    pub fn new(input: &DeriveInput) -> Result<Self> {
        let attrs = ContainerAttrs::new(input)?;

        let data = match &input.data {
            Data::Struct(DataStruct { fields, .. }) => {
                ContainerData::Struct(Fields::new(fields, FieldsLocation::Struct)?)
            }
            Data::Enum(DataEnum { variants, .. }) => ContainerData::Enum(
                variants
                    .iter()
                    .enumerate()
                    .map(|(i, v)| Variant::new(v, i))
                    .collect::<Result<Vec<_>>>()?,
            ),
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "Savable is only implementable for structs and enums",
                ));
            }
        };

        let generics = Self::build_generics(&input.generics, &attrs, &data);

        Ok(Self {
            ident: input.ident.clone(),
            generics,
            attrs,
            data,
        })
    }

    fn build_generics(
        generics: &Generics,
        attrs: &ContainerAttrs,
        _data: &ContainerData,
    ) -> Generics {
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
