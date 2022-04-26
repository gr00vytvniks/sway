use super::{FunctionDeclaration, TypeParameter};
use crate::{
    build_config::BuildConfig, error::*, parse_tree::CallPath, parser::Rule, type_engine::TypeInfo,
};

use sway_types::span::Span;

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ImplTrait {
    pub(crate) trait_name: CallPath,
    pub(crate) type_implementing_for: TypeInfo,
    pub(crate) type_implementing_for_span: Span,
    pub(crate) type_arguments: Vec<TypeParameter>,
    pub functions: Vec<FunctionDeclaration>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span,
}

/// An impl of methods without a trait
/// like `impl MyType { fn foo { .. } }`
#[derive(Debug, Clone)]
pub struct ImplSelf {
    pub(crate) type_implementing_for: TypeInfo,
    pub(crate) type_implementing_for_span: Span,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub functions: Vec<FunctionDeclaration>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span,
}
