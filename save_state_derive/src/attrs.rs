use syn::{Attribute, DeriveInput, Meta, NestedMeta, Result};

fn parse_savable_attr(attr: &Attribute) -> Result<Vec<NestedMeta>> {
    if !attr.path.is_ident("savable") {
        return Ok(Vec::new());
    }

    match attr.parse_meta()? {
        Meta::List(meta) => Ok(meta.nested.into_iter().collect()),
        other => Err(syn::Error::new_spanned(other, "expected #[savable(...)]")),
    }
}

pub struct ContainerAttrs {
    pub use_serde: bool,
}

impl ContainerAttrs {
    pub fn new(input: &DeriveInput) -> Result<Self> {
        let mut use_serde = false;

        for meta_item in input.attrs.iter().flat_map(parse_savable_attr).flatten() {
            match &meta_item {
                NestedMeta::Meta(Meta::Path(path)) if path.is_ident("serde") => {
                    use_serde = true;
                }
                NestedMeta::Meta(other) => {
                    return Err(syn::Error::new_spanned(other, "exected #[savable(serde)]"));
                }
                NestedMeta::Lit(lit) => {
                    return Err(syn::Error::new_spanned(
                        lit,
                        "unexpected literal string in savable attribute",
                    ));
                }
            }
        }

        Ok(Self { use_serde })
    }
}

#[derive(Clone)]
pub struct FieldAttrs {
    pub use_serde: bool,
    pub skip: bool,
}

impl FieldAttrs {
    pub fn new(input: &syn::Field) -> Result<Self> {
        let mut use_serde = false;
        let mut skip = false;

        for meta_item in input.attrs.iter().flat_map(parse_savable_attr).flatten() {
            match &meta_item {
                NestedMeta::Meta(Meta::Path(path)) if path.is_ident("serde") => {
                    if skip {
                        return Err(syn::Error::new_spanned(
                            meta_item,
                            "cannot have `skip` and `serde` at the same time",
                        ));
                    }
                    use_serde = true;
                }
                NestedMeta::Meta(Meta::Path(path)) if path.is_ident("skip") => {
                    if use_serde {
                        return Err(syn::Error::new_spanned(
                            meta_item,
                            "cannot have `skip` and `serde` at the same time",
                        ));
                    }
                    skip = true;
                }
                NestedMeta::Meta(other) => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "exected #[savable(serde)] or #[savable(skip)]",
                    ));
                }
                NestedMeta::Lit(lit) => {
                    return Err(syn::Error::new_spanned(
                        lit,
                        "unexpected literal string in savable attribute",
                    ));
                }
            }
        }

        Ok(Self { use_serde, skip })
    }
}
