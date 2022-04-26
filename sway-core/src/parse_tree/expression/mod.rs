use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{ident, literal::handle_parse_int_error, CallPath, Literal},
    parser::Rule,
    type_engine::{IntegerBits, TypeInfo},
    AstNode, AstNodeContent, CodeBlock, Declaration, TypeArgument, VariableDeclaration,
};

use sway_types::{ident::Ident, Span};

use either::Either;
use pest;
use pest::iterators::Pair;
use std::collections::VecDeque;

mod asm;
mod match_branch;
mod match_condition;
mod matcher;
mod method_name;
mod scrutinee;
mod unary_op;
pub(crate) use asm::*;
pub(crate) use match_branch::MatchBranch;
pub(crate) use match_condition::CatchAll;
pub(crate) use match_condition::MatchCondition;
use matcher::matcher;
pub(crate) use method_name::MethodName;
pub(crate) use scrutinee::{Scrutinee, StructScrutineeField};
pub(crate) use unary_op::UnaryOp;

/// Represents a parsed, but not yet type checked, [Expression](https://en.wikipedia.org/wiki/Expression_(computer_science)).
#[derive(Debug, Clone)]
pub enum Expression {
    Literal {
        value: Literal,
        span: Span,
    },
    FunctionApplication {
        name: CallPath,
        arguments: Vec<Expression>,
        type_arguments: Vec<TypeArgument>,
        span: Span,
    },
    LazyOperator {
        op: LazyOp,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        span: Span,
    },
    VariableExpression {
        name: Ident,
        span: Span,
    },
    Tuple {
        fields: Vec<Expression>,
        span: Span,
    },
    TupleIndex {
        prefix: Box<Expression>,
        index: usize,
        index_span: Span,
        span: Span,
    },
    Array {
        contents: Vec<Expression>,
        span: Span,
    },
    StructExpression {
        struct_name: CallPath,
        type_arguments: Vec<TypeArgument>,
        fields: Vec<StructExpressionField>,
        span: Span,
    },
    CodeBlock {
        contents: CodeBlock,
        span: Span,
    },
    IfExp {
        condition: Box<Expression>,
        then: Box<Expression>,
        r#else: Option<Box<Expression>>,
        span: Span,
    },
    MatchExp {
        if_exp: Box<Expression>,
        cases_covered: Vec<MatchCondition>,
        span: Span,
    },
    // separated into other struct for parsing reasons
    AsmExpression {
        span: Span,
        asm: AsmExpression,
    },
    MethodApplication {
        method_name: MethodName,
        contract_call_params: Vec<StructExpressionField>,
        arguments: Vec<Expression>,
        type_arguments: Vec<TypeArgument>,
        span: Span,
    },
    /// A _subfield expression_ is anything of the form:
    /// ```ignore
    /// <ident>.<ident>
    /// ```
    ///
    SubfieldExpression {
        prefix: Box<Expression>,
        span: Span,
        field_to_access: Ident,
    },
    /// A _delineated path_ is anything of the form:
    /// ```ignore
    /// <ident>::<ident>
    /// ```
    /// Where there are `n >= 2` idents.
    /// These could be either enum variant constructions, or they could be
    /// references to some sort of module in the module tree.
    /// For example, a reference to a module:
    /// ```ignore
    /// std::ops::add
    /// ```
    ///
    /// And, an enum declaration:
    /// ```ignore
    /// enum MyEnum {
    ///   Variant1,
    ///   Variant2
    /// }
    ///
    /// MyEnum::Variant1
    /// ```
    DelineatedPath {
        call_path: CallPath,
        args: Vec<Expression>,
        span: Span,
        type_arguments: Vec<TypeArgument>,
    },
    /// A cast of a hash to an ABI for calling a contract.
    AbiCast {
        abi_name: CallPath,
        address: Box<Expression>,
        span: Span,
    },
    ArrayIndex {
        prefix: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
    /// This variant serves as a stand-in for parsing-level match expression desugaring.
    /// Because types cannot be known at parsing-time, a desugared struct or enum gets
    /// special cased into this variant. During type checking, this variant is removed
    /// as is replaced with the corresponding field or argument access (given that the
    /// expression inside of the delayed resolution has the appropriate struct or enum
    /// type)
    DelayedMatchTypeResolution {
        variant: DelayedResolutionVariant,
        span: Span,
    },
    StorageAccess {
        field_names: Vec<Ident>,
        span: Span,
    },
    IfLet {
        scrutinee: Scrutinee,
        expr: Box<Expression>,
        then: CodeBlock,
        r#else: Option<Box<Expression>>,
        span: Span,
    },
    SizeOfVal {
        exp: Box<Expression>,
        span: Span,
    },
    BuiltinGetTypeProperty {
        builtin: BuiltinProperty,
        type_name: TypeInfo,
        type_span: Span,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinProperty {
    SizeOfType,
    IsRefType,
}

#[derive(Debug, Clone)]
pub enum DelayedResolutionVariant {
    StructField(DelayedStructFieldResolution),
    EnumVariant(DelayedEnumVariantResolution),
    TupleVariant(DelayedTupleVariantResolution),
}

/// During type checking, this gets replaced with struct field access.
#[derive(Debug, Clone)]
pub struct DelayedStructFieldResolution {
    pub exp: Box<Expression>,
    pub struct_name: Ident,
    pub field: Ident,
}

/// During type checking, this gets replaced with an if let, maybe, although that's not yet been
/// implemented.
#[derive(Debug, Clone)]
pub struct DelayedEnumVariantResolution {
    pub exp: Box<Expression>,
    pub call_path: CallPath,
    pub arg_num: usize,
}

/// During type checking, this gets replaced with tuple arg access.
#[derive(Debug, Clone)]
pub struct DelayedTupleVariantResolution {
    pub exp: Box<Expression>,
    pub elem_num: usize,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum LazyOp {
    And,
    Or,
}

impl LazyOp {
    fn from(op_variant: OpVariant) -> Self {
        match op_variant {
            OpVariant::And => Self::And,
            OpVariant::Or => Self::Or,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructExpressionField {
    pub(crate) name: Ident,
    pub(crate) value: Expression,
    pub(crate) span: Span,
}

pub(crate) fn error_recovery_exp(span: Span) -> Expression {
    Expression::Tuple {
        fields: vec![],
        span,
    }
}

impl Expression {
    pub(crate) fn core_ops_eq(arguments: Vec<Expression>, span: Span) -> Expression {
        Expression::MethodApplication {
            method_name: MethodName::FromType {
                call_path: CallPath {
                    prefixes: vec![
                        Ident::new_with_override("core", span.clone()),
                        Ident::new_with_override("ops", span.clone()),
                    ],
                    suffix: Op {
                        op_variant: OpVariant::Equals,
                        span: span.clone(),
                    }
                    .to_var_name(),
                    is_absolute: true,
                },
                type_name: None,
                type_name_span: None,
            },
            contract_call_params: vec![],
            arguments,
            type_arguments: vec![],
            span,
        }
    }

    pub(crate) fn core_ops(op: Op, arguments: Vec<Expression>, span: Span) -> Expression {
        Expression::MethodApplication {
            method_name: MethodName::FromType {
                call_path: CallPath {
                    prefixes: vec![
                        Ident::new_with_override("core", span.clone()),
                        Ident::new_with_override("ops", span.clone()),
                    ],
                    suffix: op.to_var_name(),
                    is_absolute: true,
                },
                type_name: None,
                type_name_span: None,
            },
            contract_call_params: vec![],
            arguments,
            type_arguments: vec![],
            span,
        }
    }

    pub(crate) fn span(&self) -> Span {
        use Expression::*;
        (match self {
            Literal { span, .. } => span,
            FunctionApplication { span, .. } => span,
            LazyOperator { span, .. } => span,
            VariableExpression { span, .. } => span,
            Tuple { span, .. } => span,
            TupleIndex { span, .. } => span,
            Array { span, .. } => span,
            StructExpression { span, .. } => span,
            CodeBlock { span, .. } => span,
            IfExp { span, .. } => span,
            MatchExp { span, .. } => span,
            AsmExpression { span, .. } => span,
            MethodApplication { span, .. } => span,
            SubfieldExpression { span, .. } => span,
            DelineatedPath { span, .. } => span,
            AbiCast { span, .. } => span,
            ArrayIndex { span, .. } => span,
            DelayedMatchTypeResolution { span, .. } => span,
            StorageAccess { span, .. } => span,
            IfLet { span, .. } => span,
            SizeOfVal { span, .. } => span,
            BuiltinGetTypeProperty { span, .. } => span,
        })
        .clone()
    }
}
