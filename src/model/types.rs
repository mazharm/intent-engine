//! Type system definitions and parsing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A type reference that can be a primitive, collection, or named type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum TypeRef {
    // Primitives
    String,
    Int,
    Float,
    Bool,
    Money,
    DateTime,
    Uuid,
    Bytes,

    // Collections
    Array(Box<TypeRef>),
    Map(Box<TypeRef>, Box<TypeRef>),
    Optional(Box<TypeRef>),

    // Named reference to another Type intent
    Named(String),
}

impl TypeRef {
    /// Parse a type string into a TypeRef
    pub fn parse(s: &str) -> Result<Self, TypeParseError> {
        let s = s.trim();

        // Check for collection types
        if let Some(inner) = s.strip_prefix("array<").and_then(|s| s.strip_suffix('>')) {
            let inner_type = Self::parse(inner)?;
            return Ok(TypeRef::Array(Box::new(inner_type)));
        }

        if let Some(inner) = s.strip_prefix("optional<").and_then(|s| s.strip_suffix('>')) {
            let inner_type = Self::parse(inner)?;
            return Ok(TypeRef::Optional(Box::new(inner_type)));
        }

        if let Some(inner) = s.strip_prefix("map<").and_then(|s| s.strip_suffix('>')) {
            // Split on comma, but need to handle nested types
            let (key, value) = split_map_types(inner)?;
            let key_type = Self::parse(key)?;
            let value_type = Self::parse(value)?;

            // Validate map key constraint
            if !key_type.is_valid_map_key() {
                return Err(TypeParseError::InvalidMapKey(key.to_string()));
            }

            return Ok(TypeRef::Map(Box::new(key_type), Box::new(value_type)));
        }

        // Check for primitives
        match s.to_lowercase().as_str() {
            "string" => Ok(TypeRef::String),
            "int" => Ok(TypeRef::Int),
            "float" => Ok(TypeRef::Float),
            "bool" => Ok(TypeRef::Bool),
            "money" => Ok(TypeRef::Money),
            "datetime" => Ok(TypeRef::DateTime),
            "uuid" => Ok(TypeRef::Uuid),
            "bytes" => Ok(TypeRef::Bytes),
            _ => {
                // Must be a named type reference
                if s.is_empty() {
                    return Err(TypeParseError::Empty);
                }
                // Validate it looks like an identifier
                if !s.chars().next().unwrap().is_alphabetic() {
                    return Err(TypeParseError::InvalidName(s.to_string()));
                }
                Ok(TypeRef::Named(s.to_string()))
            }
        }
    }

    /// Check if this type is valid as a map key
    pub fn is_valid_map_key(&self) -> bool {
        matches!(self, TypeRef::String | TypeRef::Int | TypeRef::Uuid)
    }

    /// Check if this type is a primitive
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            TypeRef::String
                | TypeRef::Int
                | TypeRef::Float
                | TypeRef::Bool
                | TypeRef::Money
                | TypeRef::DateTime
                | TypeRef::Uuid
                | TypeRef::Bytes
        )
    }

    /// Get the Rust type representation
    pub fn to_rust_type(&self) -> String {
        match self {
            TypeRef::String => "String".to_string(),
            TypeRef::Int => "i64".to_string(),
            TypeRef::Float => "f64".to_string(),
            TypeRef::Bool => "bool".to_string(),
            TypeRef::Money => "rust_decimal::Decimal".to_string(),
            TypeRef::DateTime => "chrono::DateTime<chrono::Utc>".to_string(),
            TypeRef::Uuid => "uuid::Uuid".to_string(),
            TypeRef::Bytes => "Vec<u8>".to_string(),
            TypeRef::Array(inner) => format!("Vec<{}>", inner.to_rust_type()),
            TypeRef::Map(k, v) => {
                format!("std::collections::HashMap<{}, {}>", k.to_rust_type(), v.to_rust_type())
            }
            TypeRef::Optional(inner) => format!("Option<{}>", inner.to_rust_type()),
            TypeRef::Named(name) => name.clone(),
        }
    }

    /// Get all named type references in this type (for dependency tracking)
    pub fn get_named_references(&self) -> Vec<&str> {
        match self {
            TypeRef::Named(name) => vec![name.as_str()],
            TypeRef::Array(inner) => inner.get_named_references(),
            TypeRef::Map(k, v) => {
                let mut refs = k.get_named_references();
                refs.extend(v.get_named_references());
                refs
            }
            TypeRef::Optional(inner) => inner.get_named_references(),
            _ => vec![],
        }
    }
}

impl std::fmt::Display for TypeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeRef::String => write!(f, "string"),
            TypeRef::Int => write!(f, "int"),
            TypeRef::Float => write!(f, "float"),
            TypeRef::Bool => write!(f, "bool"),
            TypeRef::Money => write!(f, "money"),
            TypeRef::DateTime => write!(f, "datetime"),
            TypeRef::Uuid => write!(f, "uuid"),
            TypeRef::Bytes => write!(f, "bytes"),
            TypeRef::Array(inner) => write!(f, "array<{}>", inner),
            TypeRef::Map(k, v) => write!(f, "map<{}, {}>", k, v),
            TypeRef::Optional(inner) => write!(f, "optional<{}>", inner),
            TypeRef::Named(name) => write!(f, "{}", name),
        }
    }
}

impl TryFrom<String> for TypeRef {
    type Error = TypeParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        TypeRef::parse(&s)
    }
}

impl From<TypeRef> for String {
    fn from(t: TypeRef) -> Self {
        t.to_string()
    }
}

/// Errors that can occur when parsing a type
#[derive(Debug, Clone, thiserror::Error)]
pub enum TypeParseError {
    #[error("Empty type string")]
    Empty,

    #[error("Invalid type name: {0}")]
    InvalidName(String),

    #[error("Invalid map key type: {0} (must be string, int, or uuid)")]
    InvalidMapKey(String),

    #[error("Unbalanced brackets in type: {0}")]
    UnbalancedBrackets(String),

    #[error("Missing map value type")]
    MissingMapValue,
}

/// Split map types at the comma, handling nested generics
fn split_map_types(s: &str) -> Result<(&str, &str), TypeParseError> {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                let key = s[..i].trim();
                let value = s[i + 1..].trim();
                if value.is_empty() {
                    return Err(TypeParseError::MissingMapValue);
                }
                return Ok((key, value));
            }
            _ => {}
        }
    }
    Err(TypeParseError::MissingMapValue)
}

/// A field definition within a Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    #[serde(rename = "type")]
    pub field_type: TypeRef,

    #[serde(default)]
    pub required: bool,
}

// ============================================================================
// Expression Language (for Function bodies)
// ============================================================================

/// An expression in the intent expression language
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Expression {
    /// Literal value (string, number, boolean, null)
    Literal {
        value: serde_json::Value,
    },

    /// Variable reference (e.g., "input", "context.amount")
    Variable {
        name: String,
    },

    /// Field access (e.g., expr.field)
    Field {
        expr: Box<Expression>,
        name: String,
    },

    /// Index access (e.g., expr[0] or expr["key"])
    Index {
        expr: Box<Expression>,
        index: Box<Expression>,
    },

    /// Function call (e.g., parse(s))
    Call {
        function: String,
        #[serde(default)]
        args: Vec<Expression>,
    },

    /// Method call (e.g., s.trim())
    Method {
        expr: Box<Expression>,
        name: String,
        #[serde(default)]
        args: Vec<Expression>,
    },

    /// Binary operation (e.g., a + b, x == y)
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// Unary operation (e.g., !x, -y)
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },

    /// Conditional (if-then-else)
    If {
        cond: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },

    /// Pattern matching
    Match {
        on: Box<Expression>,
        arms: Vec<MatchArm>,
    },

    /// Local variable bindings
    Let {
        bindings: Vec<LetBinding>,
        body: Box<Expression>,
    },

    /// Iteration
    For {
        var: String,
        iterable: Box<Expression>,
        body: Box<Expression>,
    },

    /// Early return
    Return {
        value: Box<Expression>,
    },

    /// Raise an error
    Raise {
        error: String,
        #[serde(default)]
        message: Option<Box<Expression>>,
    },

    /// Block of expressions (last expression is the result)
    Block {
        exprs: Vec<Expression>,
    },

    /// Struct/record construction
    Struct {
        name: String,
        fields: HashMap<String, Expression>,
    },

    /// Array/Vec construction
    Array {
        elements: Vec<Expression>,
    },

    /// Tuple construction
    Tuple {
        elements: Vec<Expression>,
    },

    /// Closure/lambda
    Closure {
        params: Vec<String>,
        body: Box<Expression>,
    },

    /// Try expression (? operator equivalent)
    Try {
        expr: Box<Expression>,
    },

    /// Unwrap with default
    UnwrapOr {
        expr: Box<Expression>,
        default: Box<Expression>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    #[serde(rename = "+")]
    Add,
    #[serde(rename = "-")]
    Sub,
    #[serde(rename = "*")]
    Mul,
    #[serde(rename = "/")]
    Div,
    #[serde(rename = "%")]
    Mod,

    // Comparison
    #[serde(rename = "==")]
    Eq,
    #[serde(rename = "!=")]
    Ne,
    #[serde(rename = "<")]
    Lt,
    #[serde(rename = "<=")]
    Le,
    #[serde(rename = ">")]
    Gt,
    #[serde(rename = ">=")]
    Ge,

    // Logical
    #[serde(rename = "&&")]
    And,
    #[serde(rename = "||")]
    Or,

    // String
    #[serde(rename = "++")]
    Concat,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    #[serde(rename = "!")]
    Not,
    #[serde(rename = "-")]
    Neg,
}

/// A match arm with pattern and body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    #[serde(default)]
    pub guard: Option<Box<Expression>>,
    pub body: Box<Expression>,
}

/// Pattern for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Pattern {
    /// Matches any value, ignores it
    Wildcard,

    /// Matches and binds to a variable
    Variable {
        name: String,
    },

    /// Matches a literal value
    Literal {
        value: serde_json::Value,
    },

    /// Matches an enum variant
    Variant {
        enum_name: Option<String>,
        variant: String,
        #[serde(default)]
        bindings: Vec<String>,
    },

    /// Matches a struct with specific fields
    Struct {
        name: String,
        fields: HashMap<String, Pattern>,
    },

    /// Matches a tuple
    Tuple {
        elements: Vec<Pattern>,
    },

    /// Matches an array with specific elements
    Array {
        elements: Vec<Pattern>,
        #[serde(default)]
        rest: Option<String>,
    },

    /// Matches string starting with prefix
    StartsWith {
        prefix: String,
    },

    /// Matches string ending with suffix
    EndsWith {
        suffix: String,
    },

    /// Matches Option::Some
    Some {
        binding: String,
    },

    /// Matches Option::None
    None,

    /// Matches Result::Ok
    Ok {
        binding: String,
    },

    /// Matches Result::Err
    Err {
        binding: String,
    },

    /// Or pattern (matches if any sub-pattern matches)
    Or {
        patterns: Vec<Pattern>,
    },
}

/// A let binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LetBinding {
    pub name: String,
    #[serde(default)]
    pub type_annotation: Option<String>,
    pub value: Expression,
}

/// Spec for Type intent kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSpec {
    pub fields: HashMap<String, FieldDef>,
}

impl TypeSpec {
    /// Parse from a JSON value
    pub fn from_value(value: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value.clone())
    }

    /// Get all type references used in this spec
    pub fn get_type_references(&self) -> Vec<&str> {
        self.fields
            .values()
            .flat_map(|f| f.field_type.get_named_references())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_primitives() {
        assert_eq!(TypeRef::parse("string").unwrap(), TypeRef::String);
        assert_eq!(TypeRef::parse("int").unwrap(), TypeRef::Int);
        assert_eq!(TypeRef::parse("float").unwrap(), TypeRef::Float);
        assert_eq!(TypeRef::parse("bool").unwrap(), TypeRef::Bool);
        assert_eq!(TypeRef::parse("money").unwrap(), TypeRef::Money);
        assert_eq!(TypeRef::parse("datetime").unwrap(), TypeRef::DateTime);
        assert_eq!(TypeRef::parse("uuid").unwrap(), TypeRef::Uuid);
        assert_eq!(TypeRef::parse("bytes").unwrap(), TypeRef::Bytes);
    }

    #[test]
    fn test_parse_array() {
        let t = TypeRef::parse("array<string>").unwrap();
        assert_eq!(t, TypeRef::Array(Box::new(TypeRef::String)));
    }

    #[test]
    fn test_parse_optional() {
        let t = TypeRef::parse("optional<int>").unwrap();
        assert_eq!(t, TypeRef::Optional(Box::new(TypeRef::Int)));
    }

    #[test]
    fn test_parse_map() {
        let t = TypeRef::parse("map<string, int>").unwrap();
        assert_eq!(
            t,
            TypeRef::Map(Box::new(TypeRef::String), Box::new(TypeRef::Int))
        );
    }

    #[test]
    fn test_parse_named() {
        let t = TypeRef::parse("RefundRequest").unwrap();
        assert_eq!(t, TypeRef::Named("RefundRequest".to_string()));
    }

    #[test]
    fn test_parse_nested() {
        let t = TypeRef::parse("array<optional<string>>").unwrap();
        assert_eq!(
            t,
            TypeRef::Array(Box::new(TypeRef::Optional(Box::new(TypeRef::String))))
        );
    }

    #[test]
    fn test_invalid_map_key() {
        let result = TypeRef::parse("map<float, string>");
        assert!(result.is_err());
    }

    #[test]
    fn test_to_rust_type() {
        assert_eq!(TypeRef::String.to_rust_type(), "String");
        assert_eq!(TypeRef::Int.to_rust_type(), "i64");
        assert_eq!(
            TypeRef::Array(Box::new(TypeRef::String)).to_rust_type(),
            "Vec<String>"
        );
        assert_eq!(
            TypeRef::Optional(Box::new(TypeRef::Int)).to_rust_type(),
            "Option<i64>"
        );
    }
}
