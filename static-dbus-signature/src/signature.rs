use core::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Signature {
    // Simple types
    U8,
    Bool,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F64,
    Str,
    Signature,
    ObjectPath,
    Value,
    #[cfg(unix)]
    Fd,

    // Container types
    Array(ChildSignature),
    Dict {
        key: ChildSignature,
        value: ChildSignature,
    },
    Structure(FieldsSignatures),
    #[cfg(feature = "gvariant")]
    Maybe(ChildSignature),
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Signature::U8 => write!(f, "y"),
            Signature::Bool => write!(f, "b"),
            Signature::I16 => write!(f, "n"),
            Signature::U16 => write!(f, "q"),
            Signature::I32 => write!(f, "i"),
            Signature::U32 => write!(f, "u"),
            Signature::I64 => write!(f, "x"),
            Signature::U64 => write!(f, "t"),
            Signature::F64 => write!(f, "d"),
            Signature::Str => write!(f, "s"),
            Signature::Signature => write!(f, "g"),
            Signature::ObjectPath => write!(f, "o"),
            Signature::Value => write!(f, "v"),
            #[cfg(unix)]
            Signature::Fd => write!(f, "h"),
            Signature::Array(array) => write!(f, "a{}", **array),
            Signature::Dict { key, value } => {
                write!(f, "a{{")?;

                key.fmt(f)?;
                value.fmt(f)?;

                write!(f, "}}")
            }
            Signature::Structure(structure) => {
                write!(f, "(")?;
                for field in structure.iter() {
                    field.fmt(f)?;
                }
                write!(f, ")")
            }
            #[cfg(feature = "gvariant")]
            Signature::Maybe(maybe) => write!(f, "m{}", **maybe),
        }
    }
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Signature::U8, Signature::U8)
            | (Signature::Bool, Signature::Bool)
            | (Signature::I16, Signature::I16)
            | (Signature::U16, Signature::U16)
            | (Signature::I32, Signature::I32)
            | (Signature::U32, Signature::U32)
            | (Signature::I64, Signature::I64)
            | (Signature::U64, Signature::U64)
            | (Signature::F64, Signature::F64)
            | (Signature::Str, Signature::Str)
            | (Signature::Signature, Signature::Signature)
            | (Signature::ObjectPath, Signature::ObjectPath)
            | (Signature::Value, Signature::Value)
            | (Signature::Fd, Signature::Fd) => true,
            (Signature::Array(a), Signature::Array(b)) => a.eq(b),
            (
                Signature::Dict {
                    key: key_a,
                    value: value_a,
                },
                Signature::Dict {
                    key: key_b,
                    value: value_b,
                },
            ) => key_a.eq(key_b) && value_a.eq(value_b),
            (Signature::Structure(a), Signature::Structure(b)) => a.iter().eq(b.iter()),
            #[cfg(feature = "gvariant")]
            (Signature::Maybe(a), Signature::Maybe(b)) => a.eq(b),
            _ => false,
        }
    }
}

impl Eq for Signature {}

impl PartialOrd for Signature {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Signature::U8, Signature::U8)
            | (Signature::Bool, Signature::Bool)
            | (Signature::I16, Signature::I16)
            | (Signature::U16, Signature::U16)
            | (Signature::I32, Signature::I32)
            | (Signature::U32, Signature::U32)
            | (Signature::I64, Signature::I64)
            | (Signature::U64, Signature::U64)
            | (Signature::F64, Signature::F64)
            | (Signature::Str, Signature::Str)
            | (Signature::Signature, Signature::Signature)
            | (Signature::ObjectPath, Signature::ObjectPath)
            | (Signature::Value, Signature::Value)
            | (Signature::Fd, Signature::Fd) => Some(std::cmp::Ordering::Equal),
            (Signature::Array(a), Signature::Array(b)) => a.partial_cmp(b),
            (
                Signature::Dict {
                    key: key_a,
                    value: value_a,
                },
                Signature::Dict {
                    key: key_b,
                    value: value_b,
                },
            ) => match key_a.partial_cmp(key_b) {
                Some(std::cmp::Ordering::Equal) => value_a.partial_cmp(value_b),
                other => other,
            },
            (Signature::Structure(a), Signature::Structure(b)) => a.iter().partial_cmp(b.iter()),
            #[cfg(feature = "gvariant")]
            (Signature::Maybe(a), Signature::Maybe(b)) => a.partial_cmp(b),
            (a, b) => None,
        }
    }
}

impl Ord for Signature {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum FieldsSignatures {
    Static {
        fields: &'static [&'static Signature],
    },
    Dynamic {
        fields: Arc<[Signature]>,
    },
}

impl FieldsSignatures {
    pub fn iter(&self) -> impl Iterator<Item = &Signature> {
        use std::slice::Iter;

        enum Fields<'a> {
            Static(Iter<'static, &'static Signature>),
            Dynamic(Iter<'a, Signature>),
        }

        impl<'a> Iterator for Fields<'a> {
            type Item = &'a Signature;

            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Fields::Static(iter) => iter.next().map(|&f| f),
                    Fields::Dynamic(iter) => iter.next(),
                }
            }
        }

        match self {
            FieldsSignatures::Static { fields } => Fields::Static(fields.iter()),
            FieldsSignatures::Dynamic { fields } => Fields::Dynamic(fields.iter()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChildSignature {
    Static { child: &'static Signature },
    Dynamic { child: Arc<Signature> },
}

impl Deref for ChildSignature {
    type Target = Signature;

    fn deref(&self) -> &Self::Target {
        match self {
            ChildSignature::Static { child } => child,
            ChildSignature::Dynamic { child } => child,
        }
    }
}
