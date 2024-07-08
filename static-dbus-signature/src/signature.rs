use core::fmt;
use std::fmt::{Display, Formatter};
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
    Array(ArraySignature),
    Dict(DictSignature),
    Structure(StructSignature),
    #[cfg(feature = "gvariant")]
    Maybe(MaybeSignature),
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
            Signature::Array(array) => write!(f, "a{}", array.child()),
            Signature::Dict(dict) => {
                write!(f, "a{{")?;

                dict.key().fmt(f)?;
                dict.value().fmt(f)?;

                write!(f, "}}")
            }
            Signature::Structure(structure) => {
                write!(f, "(")?;
                for field in structure.fields() {
                    field.fmt(f)?;
                }
                write!(f, ")")
            }
            #[cfg(feature = "gvariant")]
            Signature::Maybe(maybe) => write!(f, "m{}", maybe.child()),
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
            (Signature::Array(a), Signature::Array(b)) => a.child() == b.child(),
            (Signature::Dict(a), Signature::Dict(b)) => {
                a.key() == b.key() && a.value() == b.value()
            }
            (Signature::Structure(a), Signature::Structure(b)) => a.fields().eq(b.fields()),
            #[cfg(feature = "gvariant")]
            (Signature::Maybe(a), Signature::Maybe(b)) => a.child() == b.child(),
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
            (Signature::Array(a), Signature::Array(b)) => a.child().partial_cmp(b.child()),
            (Signature::Dict(a), Signature::Dict(b)) => match a.key().partial_cmp(b.key()) {
                Some(std::cmp::Ordering::Equal) => a.value().partial_cmp(b.value()),
                other => other,
            },
            (Signature::Structure(a), Signature::Structure(b)) => {
                a.fields().partial_cmp(b.fields())
            }
            #[cfg(feature = "gvariant")]
            (Signature::Maybe(a), Signature::Maybe(b)) => a.child().partial_cmp(b.child()),
            (a, b) => a.to_string().partial_cmp(&b.to_string()),
        }
    }
}

impl Ord for Signature {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum StructSignature {
    Static {
        fields: &'static [&'static Signature],
    },
    Dynamic {
        fields: Arc<[Signature]>,
    },
}

impl StructSignature {
    pub fn fields(&self) -> impl Iterator<Item = &Signature> {
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
            StructSignature::Static { fields } => Fields::Static(fields.iter()),
            StructSignature::Dynamic { fields } => Fields::Dynamic(fields.iter()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ArraySignature {
    Static { child: &'static Signature },
    Dynamic { child: Arc<Signature> },
}

impl ArraySignature {
    pub fn child(&self) -> &Signature {
        match self {
            ArraySignature::Static { child } => child,
            ArraySignature::Dynamic { child } => child,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DictSignature {
    Static {
        key: &'static Signature,
        value: &'static Signature,
    },
    Dynamic {
        key: Arc<Signature>,
        value: Arc<Signature>,
    },
}

impl DictSignature {
    pub fn key(&self) -> &Signature {
        match self {
            DictSignature::Static { key, .. } => key,
            DictSignature::Dynamic { key, .. } => key,
        }
    }

    pub fn value(&self) -> &Signature {
        match self {
            DictSignature::Static { value, .. } => value,
            DictSignature::Dynamic { value, .. } => value,
        }
    }
}

#[cfg(feature = "gvariant")]
#[derive(Debug, Clone)]
pub enum MaybeSignature {
    Static { child: &'static Signature },
    Dynamic { child: Arc<Signature> },
}

#[cfg(feature = "gvariant")]
impl MaybeSignature {
    pub fn child(&self) -> &Signature {
        match self {
            MaybeSignature::Static { child } => child,
            MaybeSignature::Dynamic { child } => child,
        }
    }
}
