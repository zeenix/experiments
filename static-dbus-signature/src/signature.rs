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

#[derive(Debug, Clone, Eq, Ord)]
pub enum StructSignature {
    Static {
        fields: &'static [&'static Signature],
    },
    Dynamic {
        fields: Vec<Signature>,
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

impl PartialEq for StructSignature {
    fn eq(&self, other: &Self) -> bool {
        self.fields().eq(other.fields())
    }
}

impl PartialOrd for StructSignature {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.fields().partial_cmp(other.fields())
    }
}

#[derive(Debug, Clone, Eq, Ord)]
pub enum ArraySignature {
    Static { child: &'static Signature },
    Dynamic { child: Box<Signature> },
}

impl PartialEq for ArraySignature {
    fn eq(&self, other: &Self) -> bool {
        self.child() == other.child()
    }
}

impl PartialOrd for ArraySignature {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.child().partial_cmp(other.child())
    }
}

impl ArraySignature {
    pub fn child(&self) -> &Signature {
        match self {
            ArraySignature::Static { child } => child,
            ArraySignature::Dynamic { child } => child,
        }
    }
}

#[derive(Debug, Clone, Eq, Ord)]
pub enum DictSignature {
    Static {
        key: &'static Signature,
        value: &'static Signature,
    },
    Dynamic {
        key: Box<Signature>,
        value: Box<Signature>,
    },
}

impl PartialEq for DictSignature {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key() && self.value() == other.value()
    }
}

impl PartialOrd for DictSignature {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.key().partial_cmp(other.key()) {
            Some(std::cmp::Ordering::Equal) => self.value().partial_cmp(other.value()),
            other => other,
        }
    }
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
#[derive(Debug, Clone, Eq, Ord)]
pub enum MaybeSignature {
    Static { child: &'static Signature },
    Dynamic { child: Box<Signature> },
}

#[cfg(feature = "gvariant")]
impl PartialEq for MaybeSignature {
    fn eq(&self, other: &Self) -> bool {
        self.child() == other.child()
    }
}

#[cfg(feature = "gvariant")]
impl PartialOrd for MaybeSignature {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.child().partial_cmp(other.child())
    }
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
