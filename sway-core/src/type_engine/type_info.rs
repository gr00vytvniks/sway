use super::*;

use crate::{
    build_config::BuildConfig,
    semantic_analysis::ast_node::{TypedEnumVariant, TypedStructField},
    CallPath, Ident, Rule, TypeArgument, TypeParameter,
};

use sway_types::span::Span;

use derivative::Derivative;
use pest::iterators::Pair;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum AbiName {
    Deferred,
    Known(CallPath),
}

impl std::fmt::Display for AbiName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &(match self {
                AbiName::Deferred => "for unspecified ABI".to_string(),
                AbiName::Known(cp) => cp.to_string(),
            }),
        )
    }
}
/// Type information without an associated value, used for type inferencing and definition.
// TODO use idents instead of Strings when we have arena spans
#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub enum TypeInfo {
    Unknown,
    UnknownGeneric {
        name: Ident,
    },
    Str(u64),
    UnsignedInteger(IntegerBits),
    Enum {
        name: Ident,
        variant_types: Vec<TypedEnumVariant>,
    },
    Struct {
        name: Ident,
        fields: Vec<TypedStructField>,
    },
    Boolean,
    /// For the type inference engine to use when a type references another type
    Ref(TypeId),

    Tuple(Vec<TypeArgument>),
    /// Represents a type which contains methods to issue a contract call.
    /// The specific contract is identified via the `Ident` within.
    ContractCaller {
        abi_name: AbiName,
        // this is raw source code to be evaluated later.
        // TODO(static span): we can just use `TypedExpression` here or something more elegant
        // `TypedExpression` requires implementing a lot of `Hash` all over the place, not the
        // best...
        address: String,
    },
    /// A custom type could be a struct or similar if the name is in scope,
    /// or just a generic parameter if it is not.
    /// At parse time, there is no sense of scope, so this determination is not made
    /// until the semantic analysis stage.
    Custom {
        name: Ident,
        type_arguments: Vec<TypeArgument>,
    },
    SelfType,
    Byte,
    B256,
    /// This means that specific type of a number is not yet known. It will be
    /// determined via inference at a later time.
    Numeric,
    Contract,
    // used for recovering from errors in the ast
    ErrorRecovery,
    // Static, constant size arrays.
    Array(TypeId, usize),
    /// Represents the entire storage declaration struct
    /// Stored without initializers here, as typed struct fields,
    /// so type checking is able to treat it as a struct with fields.
    Storage {
        fields: Vec<TypedStructField>,
    },
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TypeInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TypeInfo::Str(len) => {
                state.write_u8(1);
                len.hash(state);
            }
            TypeInfo::UnsignedInteger(bits) => {
                state.write_u8(2);
                bits.hash(state);
            }
            TypeInfo::Numeric => {
                state.write_u8(3);
            }
            TypeInfo::Boolean => {
                state.write_u8(4);
            }
            TypeInfo::Tuple(fields) => {
                state.write_u8(5);
                fields.hash(state);
            }
            TypeInfo::Byte => {
                state.write_u8(6);
            }
            TypeInfo::B256 => {
                state.write_u8(7);
            }
            TypeInfo::Enum {
                name,
                variant_types,
            } => {
                state.write_u8(8);
                name.hash(state);
                variant_types.hash(state);
            }
            TypeInfo::Struct { name, fields } => {
                state.write_u8(9);
                name.hash(state);
                fields.hash(state);
            }
            TypeInfo::ContractCaller { abi_name, address } => {
                state.write_u8(10);
                abi_name.hash(state);
                address.hash(state);
            }
            TypeInfo::Contract => {
                state.write_u8(11);
            }
            TypeInfo::ErrorRecovery => {
                state.write_u8(12);
            }
            TypeInfo::Unknown => {
                state.write_u8(13);
            }
            TypeInfo::SelfType => {
                state.write_u8(14);
            }
            TypeInfo::UnknownGeneric { name } => {
                state.write_u8(15);
                name.hash(state);
            }
            TypeInfo::Custom {
                name,
                type_arguments,
            } => {
                state.write_u8(16);
                name.hash(state);
                type_arguments.hash(state);
            }
            TypeInfo::Ref(id) => {
                state.write_u8(17);
                look_up_type_id(*id).hash(state);
            }
            TypeInfo::Array(elem_ty, count) => {
                state.write_u8(18);
                look_up_type_id(*elem_ty).hash(state);
                count.hash(state);
            }
            TypeInfo::Storage { fields } => {
                state.write_u8(19);
                fields.hash(state);
            }
        }
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unknown, Self::Unknown) => true,
            (Self::Boolean, Self::Boolean) => true,
            (Self::SelfType, Self::SelfType) => true,
            (Self::Byte, Self::Byte) => true,
            (Self::B256, Self::B256) => true,
            (Self::Numeric, Self::Numeric) => true,
            (Self::Contract, Self::Contract) => true,
            (Self::ErrorRecovery, Self::ErrorRecovery) => true,
            (Self::UnknownGeneric { name: l }, Self::UnknownGeneric { name: r }) => l == r,
            (
                Self::Custom {
                    name: l_name,
                    type_arguments: l_type_args,
                },
                Self::Custom {
                    name: r_name,
                    type_arguments: r_type_args,
                },
            ) => l_name == r_name && l_type_args == r_type_args,
            (Self::Str(l), Self::Str(r)) => l == r,
            (Self::UnsignedInteger(l), Self::UnsignedInteger(r)) => l == r,
            (
                Self::Enum {
                    name: l_name,
                    variant_types: l_variant_types,
                    ..
                },
                Self::Enum {
                    name: r_name,
                    variant_types: r_variant_types,
                    ..
                },
            ) => l_name == r_name && l_variant_types == r_variant_types,
            (
                Self::Struct {
                    name: l_name,
                    fields: l_fields,
                    ..
                },
                Self::Struct {
                    name: r_name,
                    fields: r_fields,
                    ..
                },
            ) => l_name == r_name && l_fields == r_fields,
            (Self::Ref(l), Self::Ref(r)) => look_up_type_id(*l) == look_up_type_id(*r),
            (Self::Tuple(l), Self::Tuple(r)) => l
                .iter()
                .zip(r.iter())
                .map(|(l, r)| look_up_type_id(l.type_id) == look_up_type_id(r.type_id))
                .all(|x| x),
            (
                Self::ContractCaller {
                    abi_name: l_abi_name,
                    address: l_address,
                },
                Self::ContractCaller {
                    abi_name: r_abi_name,
                    address: r_address,
                },
            ) => l_abi_name == r_abi_name && l_address == r_address,
            (Self::Array(l0, l1), Self::Array(r0, r1)) => {
                look_up_type_id(*l0) == look_up_type_id(*r0) && l1 == r1
            }
            (TypeInfo::Storage { fields: l_fields }, TypeInfo::Storage { fields: r_fields }) => {
                l_fields == r_fields
            }
            _ => false,
        }
    }
}

impl Eq for TypeInfo {}

impl Default for TypeInfo {
    fn default() -> Self {
        TypeInfo::Unknown
    }
}
