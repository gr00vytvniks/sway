use crate::{
    build_config::BuildConfig,
    error::{err, ok, CompileResult, ParserLifter, Warning},
    parse_tree::{ident, Expression, Visibility},
    parser::Rule,
    style::is_screaming_snake_case,
    type_engine::TypeInfo,
};

use sway_types::{ident::Ident, span::Span};

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ConstantDeclaration {
    pub name: Ident,
    pub type_ascription: TypeInfo,
    pub value: Expression,
    pub visibility: Visibility,
}
