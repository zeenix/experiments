use core::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
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

impl FromStr for Signature {
    type Err = ();

    // Let's use nom to parse the signature
    fn from_str(s: &str) -> Result<Self, ()> {
        use nom::branch::alt;
        use nom::bytes::complete::tag;
        use nom::character::complete::{char, one_of};
        use nom::combinator::map;
        use nom::multi::many1;
        use nom::sequence::{delimited, pair, preceded, terminated};

        fn parse_signature(s: &str) -> nom::IResult<&str, Signature> {
            let simple_type = alt((
                map(tag("y"), |_| Signature::U8),
                map(tag("b"), |_| Signature::Bool),
                map(tag("n"), |_| Signature::I16),
                map(tag("q"), |_| Signature::U16),
                map(tag("i"), |_| Signature::I32),
                map(tag("u"), |_| Signature::U32),
                map(tag("x"), |_| Signature::I64),
                map(tag("t"), |_| Signature::U64),
                map(tag("d"), |_| Signature::F64),
                map(tag("s"), |_| Signature::Str),
                map(tag("g"), |_| Signature::Signature),
                map(tag("o"), |_| Signature::ObjectPath),
                map(tag("v"), |_| Signature::Value),
                #[cfg(unix)]
                map(tag("h"), |_| Signature::Fd),
            ));

            let array = map(pair(char('a'), parse_signature), |(_, child)| {
                Signature::Array(child.into())
            });

            let dict = map(
                delimited(char('a'), pair(parse_signature, parse_signature), char('}')),
                |(key, value)| Signature::Dict {
                    key: key.into(),
                    value: value.into(),
                },
            );

            let structure = map(
                delimited(char('('), many1(parse_signature), char(')')),
                |fields| {
                    Signature::Structure(FieldsSignatures::Dynamic {
                        fields: fields.into(),
                    })
                },
            );

            #[cfg(feature = "gvariant")]
            let maybe = map(pair(char('m'), parse_signature), |(_, child)| {
                Signature::Maybe(child.into())
            });

            alt((
                simple_type,
                array,
                dict,
                structure,
                #[cfg(feature = "gvariant")]
                maybe,
            ))(s)
        }

        let (_, signature) = parse_signature(s).map_err(|_| ())?;

        Ok(signature)
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

impl From<Arc<[Signature]>> for FieldsSignatures {
    fn from(fields: Arc<[Signature]>) -> Self {
        FieldsSignatures::Dynamic { fields }
    }
}

impl From<Vec<Signature>> for FieldsSignatures {
    fn from(fields: Vec<Signature>) -> Self {
        FieldsSignatures::Dynamic {
            fields: fields.into(),
        }
    }
}

impl From<&'static [&'static Signature]> for FieldsSignatures {
    fn from(fields: &'static [&'static Signature]) -> Self {
        FieldsSignatures::Static { fields }
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

impl From<Arc<Signature>> for ChildSignature {
    fn from(child: Arc<Signature>) -> Self {
        ChildSignature::Dynamic { child }
    }
}

impl From<Signature> for ChildSignature {
    fn from(child: Signature) -> Self {
        ChildSignature::Dynamic {
            child: Arc::new(child),
        }
    }
}

impl From<&'static Signature> for ChildSignature {
    fn from(child: &'static Signature) -> Self {
        ChildSignature::Static { child }
    }
}
