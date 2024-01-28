use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use std::collections::HashSet;
use syn::{
    parse_quote, Data, DataEnum, DataStruct, DeriveInput, Generics, Ident, Index, Result, Type,
    WherePredicate,
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
            .filter(|&f| !f.attrs.skip)
            .cloned()
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
        data: &ContainerData,
    ) -> Generics {
        // taken from `serde_derive`
        struct FindTyParams<'ast> {
            // Set of all generic type parameters on the current struct (A, B, C in
            // the example). Initialized up front.
            all_type_params: &'ast HashSet<syn::Ident>,

            // Set of generic type parameters used in fields for which filter
            // returns true (A and B in the example). Filled in as the visitor sees
            // them.
            relevant_type_params: HashSet<syn::Ident>,

            // Fields whose type is an associated type of one of the generic type
            // parameters.
            associated_type_usage: Vec<syn::TypePath>,
        }

        impl<'ast> FindTyParams<'ast> {
            fn visit_path(&mut self, path: &syn::Path) {
                if let Some(seg) = path.segments.last() {
                    if seg.ident == "PhantomData" {
                        // Hardcoded exception for phantom data
                        return;
                    }
                }
                if path.leading_colon.is_none() && path.segments.len() == 1 {
                    let id = &path.segments[0].ident;
                    if self.all_type_params.contains(id) {
                        self.relevant_type_params.insert(id.clone());
                    }
                }

                for segment in &path.segments {
                    self.visit_path_segment(segment);
                }
            }

            // Everything below is simply traversing the syntax tree.

            fn visit_type(&mut self, ty: &syn::Type) {
                match ty {
                    syn::Type::Array(ty) => self.visit_type(&ty.elem),
                    syn::Type::BareFn(ty) => {
                        for arg in &ty.inputs {
                            self.visit_type(&arg.ty);
                        }
                        self.visit_return_type(&ty.output);
                    }
                    syn::Type::Group(ty) => self.visit_type(&ty.elem),
                    syn::Type::ImplTrait(ty) => {
                        for bound in &ty.bounds {
                            self.visit_type_param_bound(bound);
                        }
                    }
                    syn::Type::Paren(ty) => self.visit_type(&ty.elem),
                    syn::Type::Path(ty) => {
                        if let Some(syn::punctuated::Pair::Punctuated(t, _)) =
                            ty.path.segments.pairs().next()
                        {
                            if self.all_type_params.contains(&t.ident) {
                                self.associated_type_usage.push(ty.clone());
                            }
                        }

                        if let Some(qself) = &ty.qself {
                            self.visit_type(&qself.ty);
                        }
                        self.visit_path(&ty.path);
                    }
                    syn::Type::Ptr(ty) => self.visit_type(&ty.elem),
                    syn::Type::Reference(ty) => self.visit_type(&ty.elem),
                    syn::Type::Slice(ty) => self.visit_type(&ty.elem),
                    syn::Type::TraitObject(ty) => {
                        for bound in &ty.bounds {
                            self.visit_type_param_bound(bound);
                        }
                    }
                    syn::Type::Tuple(ty) => {
                        for elem in &ty.elems {
                            self.visit_type(elem);
                        }
                    }

                    syn::Type::Macro(_)
                    | syn::Type::Infer(_)
                    | syn::Type::Never(_)
                    | syn::Type::Verbatim(_) => {}
                    _ => {}
                }
            }

            fn visit_path_segment(&mut self, segment: &syn::PathSegment) {
                self.visit_path_arguments(&segment.arguments);
            }

            fn visit_path_arguments(&mut self, arguments: &syn::PathArguments) {
                match arguments {
                    syn::PathArguments::None => {}
                    syn::PathArguments::AngleBracketed(arguments) => {
                        for arg in &arguments.args {
                            match arg {
                                syn::GenericArgument::Type(arg) => self.visit_type(arg),
                                syn::GenericArgument::Binding(arg) => self.visit_type(&arg.ty),
                                syn::GenericArgument::Lifetime(_)
                                | syn::GenericArgument::Constraint(_)
                                | syn::GenericArgument::Const(_) => {}
                            }
                        }
                    }
                    syn::PathArguments::Parenthesized(arguments) => {
                        for argument in &arguments.inputs {
                            self.visit_type(argument);
                        }
                        self.visit_return_type(&arguments.output);
                    }
                }
            }

            fn visit_return_type(&mut self, return_type: &syn::ReturnType) {
                match return_type {
                    syn::ReturnType::Default => {}
                    syn::ReturnType::Type(_, output) => self.visit_type(output),
                }
            }

            fn visit_type_param_bound(&mut self, bound: &syn::TypeParamBound) {
                match bound {
                    syn::TypeParamBound::Trait(bound) => self.visit_path(&bound.path),
                    syn::TypeParamBound::Lifetime(_) => {}
                }
            }

            fn generate_bounds(self, bounds: Vec<syn::TypeParamBound>) -> Vec<WherePredicate> {
                self.relevant_type_params
                    .into_iter()
                    .map(|id| syn::TypePath {
                        qself: None,
                        path: id.into(),
                    })
                    .chain(self.associated_type_usage)
                    .map::<WherePredicate, _>(|bounded_ty| parse_quote!(#bounded_ty: #(#bounds)+*))
                    .collect()
            }
        }

        let all_type_params = &generics
            .type_params()
            .map(|param| param.ident.clone())
            .collect::<HashSet<_>>();

        let mut savable_visitor = FindTyParams {
            all_type_params,
            relevant_type_params: HashSet::new(),
            associated_type_usage: Vec::new(),
        };
        let mut serde_visitor = FindTyParams {
            all_type_params,
            relevant_type_params: HashSet::new(),
            associated_type_usage: Vec::new(),
        };

        match &data {
            ContainerData::Struct(fields) => {
                if attrs.use_serde {
                    fields
                        .unskipped_fields
                        .iter()
                        .for_each(|f| serde_visitor.visit_type(&f.ty))
                } else {
                    fields
                        .unskipped_fields
                        .iter()
                        .filter(|f| !f.attrs.use_serde)
                        .for_each(|f| savable_visitor.visit_type(&f.ty));

                    fields
                        .unskipped_fields
                        .iter()
                        .filter(|f| f.attrs.use_serde)
                        .for_each(|f| serde_visitor.visit_type(&f.ty));
                }
            }
            ContainerData::Enum(variants) => {
                let all_fields = variants.iter().flat_map(|v| &v.fields.unskipped_fields);

                if attrs.use_serde {
                    all_fields.for_each(|f| serde_visitor.visit_type(&f.ty));
                } else {
                    all_fields
                        .clone()
                        .filter(|f| !f.attrs.use_serde)
                        .for_each(|f| savable_visitor.visit_type(&f.ty));

                    all_fields
                        .filter(|f| f.attrs.use_serde)
                        .for_each(|f| serde_visitor.visit_type(&f.ty));
                }
            }
        }

        let mut generics = generics.clone();
        let predicates = &mut generics.make_where_clause().predicates;
        predicates
            .extend(savable_visitor.generate_bounds(vec![parse_quote!(::save_state::Savable)]));
        predicates.extend(serde_visitor.generate_bounds(vec![
            parse_quote!(::serde::Serialize),
            parse_quote!(::serde::de::DeserializeOwned),
        ]));
        generics
    }
}
