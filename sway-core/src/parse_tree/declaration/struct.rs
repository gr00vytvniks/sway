use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{declaration::TypeParameter, ident, Visibility},
    parser::Rule,
    style::{is_snake_case, is_upper_camel_case},
    type_engine::TypeInfo,
};

use sway_types::{ident::Ident, span::Span};

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct StructDeclaration {
    pub name: Ident,
    pub(crate) fields: Vec<StructField>,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub visibility: Visibility,
    pub(crate) span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct StructField {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeInfo,
    pub(crate) span: Span,
    pub(crate) type_span: Span,
}
