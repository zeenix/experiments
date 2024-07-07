use core::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    Array {
        child: &'static Signature,
    },
    Dict {
        key: &'static Signature,
        value: &'static Signature,
    },
    Structure(StructSignature),
    #[cfg(feature = "gvariant")]
    Maybe {
        child: &'static Signature,
    },
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
            Signature::Array { child } => write!(f, "a{}", child),
            Signature::Dict { key, value } => {
                write!(f, "a{{")?;

                key.fmt(f)?;
                value.fmt(f)?;

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
            Signature::Maybe { child } => write!(f, "m{}", child),
        }
    }
}

// FIXME: Ensure both variants are considered equal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StructSignature {
    Static(&'static [&'static Signature]),
    Dynamic(Vec<Signature>),
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
            StructSignature::Static(fields) => Fields::Static(fields.iter()),
            StructSignature::Dynamic(fields) => Fields::Dynamic(fields.iter()),
        }
    }
}
