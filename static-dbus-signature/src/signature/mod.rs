mod child;
pub use child::ChildSignature;
mod fields;
pub use fields::FieldsSignatures;

use core::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use crate::r#type::Type;
use crate::signature;

#[derive(Debug, Clone)]
pub enum Signature {
    // Basic types
    Unit,
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

impl Signature {
    /// The size of the string form of `self`.
    pub fn string_len(&self) -> usize {
        match self {
            Signature::Unit => 0,
            Signature::U8
            | Signature::Bool
            | Signature::I16
            | Signature::U16
            | Signature::I32
            | Signature::U32
            | Signature::I64
            | Signature::U64
            | Signature::F64
            | Signature::Str
            | Signature::Signature
            | Signature::ObjectPath
            | Signature::Value => 1,
            #[cfg(unix)]
            Signature::Fd => 1,
            Signature::Array(child) => 1 + child.string_len(),
            Signature::Dict { key, value } => 3 + key.string_len() + value.string_len(),
            Signature::Structure(fields) => {
                let mut len = 2;
                for field in fields.iter() {
                    len += field.string_len();
                }
                len
            }
            #[cfg(feature = "gvariant")]
            Signature::Maybe(child) => 1 + child.string_len(),
        }
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Signature::Unit => write!(f, ""),
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

    fn from_str(s: &str) -> Result<Self, ()> {
        parse(s, false)
    }
}

/// Validate the given signature string.
pub fn validate(s: &str) -> Result<(), ()> {
    parse(s, true).map(|_| ())
}

/// Parse a signature string into a `Signature`.
///
/// When `check_only` is true, the function will not allocate memory for the dynamic types.
/// Instead it will return dummy values in the parsed Signature.
fn parse(s: &str, check_only: bool) -> Result<Signature, ()> {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::char;
    use nom::combinator::{all_consuming, eof, map};
    use nom::multi::{many1, many1_count};
    use nom::sequence::{delimited, pair, preceded, terminated};

    let empty = map(eof, |_| Signature::Unit);

    // `many1` allocates so we only want to use it when `check_only == false`
    fn many(
        s: &str,
        check_only: bool,
        top_level: bool,
    ) -> Result<(&str, Signature), nom::Err<nom::error::Error<&str>>> {
        let parser = |s| parse_signature(s, check_only);
        if check_only {
            return map(many1_count(parser), |_| Signature::Unit)(s);
        }

        map(many1(parser), |mut signatures| {
            if top_level {
                // On the top-level, we want to return:
                //
                // * unit signature if there are none.
                // * the signature directly if there is only one.
                if signatures.is_empty() {
                    return Signature::Unit;
                } else if signatures.len() == 1 {
                    return signatures.remove(0);
                }
            }

            Signature::Structure(FieldsSignatures::Dynamic {
                fields: signatures.into(),
            })
        })(s)
    }

    fn parse_signature(s: &str, check_only: bool) -> nom::IResult<&str, Signature> {
        let parse_with_context = |s| parse_signature(s, check_only);

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

        let dict = map(
            pair(
                char('a'),
                delimited(
                    char('{'),
                    pair(parse_with_context, parse_with_context),
                    char('}'),
                ),
            ),
            |(_, (key, value))| {
                if check_only {
                    return Signature::Dict {
                        key: <()>::SIGNATURE.into(),
                        value: <()>::SIGNATURE.into(),
                    };
                }

                Signature::Dict {
                    key: key.into(),
                    value: value.into(),
                }
            },
        );

        let array = map(pair(char('a'), parse_with_context), |(_, child)| {
            if check_only {
                return Signature::Array(<()>::SIGNATURE.into());
            }

            Signature::Array(child.into())
        });

        let structure = delimited(char('('), |s| many(s, check_only, false), char(')'));

        #[cfg(feature = "gvariant")]
        let maybe = map(pair(char('m'), parse_with_context), |(_, child)| {
            if check_only {
                return Signature::Maybe(<()>::SIGNATURE.into());
            }

            Signature::Maybe(child.into())
        });

        alt((
            simple_type,
            dict,
            array,
            structure,
            #[cfg(feature = "gvariant")]
            maybe,
        ))(s)
    }

    let (_, signature) =
        all_consuming(alt((empty, |s| many(s, check_only, true))))(s).map_err(|_| ())?;

    Ok(signature)
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Signature::Unit, Signature::Unit)
            | (Signature::U8, Signature::U8)
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
            (Signature::Unit, Signature::Unit)
            | (Signature::U8, Signature::U8)
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! validate {
        ($($signature:literal => $expected:expr),+) => {
            $(
                assert!(validate($signature).is_ok());
                let parsed = Signature::from_str($signature).unwrap();
                assert_eq!(parsed, $expected);
            )+
        };
    }

    #[test]
    fn validate_strings() {
        validate!(
            "" => Signature::Unit,
            "y" => Signature::U8,
            "b" => Signature::Bool,
            "n" => Signature::I16,
            "q" => Signature::U16,
            "i" => Signature::I32,
            "u" => Signature::U32,
            "x" => Signature::I64,
            "t" => Signature::U64,
            "d" => Signature::F64,
            "s" => Signature::Str,
            "g" => Signature::Signature,
            "o" => Signature::ObjectPath,
            "v" => Signature::Value,
            "xs" => Signature::Structure(FieldsSignatures::Static {
                fields: &[&Signature::I64, &Signature::Str]
            }),
            "(ysa{sd})" => Signature::Structure(FieldsSignatures::Static {
                fields: &[
                    &Signature::U8,
                    &Signature::Str,
                    &Signature::Dict {
                        key: ChildSignature::Static { child: &Signature::Str },
                        value: ChildSignature::Static { child: &Signature::F64 },
                    },
                ],
            }),
            "a(y)" => Signature::Array(ChildSignature::Static {
                child: &Signature::Structure(FieldsSignatures::Static { fields: &[&Signature::U8] }),
            }),
            "a{yy}" => Signature::Dict {
                key: ChildSignature::Static {
                    child: &Signature::U8
                },
                value: ChildSignature::Static {
                    child: &Signature::U8
                }
            },
            "(yy)" => Signature::Structure(FieldsSignatures::Static {
                fields: &[&Signature::U8, &Signature::U8]
            }),
            "a{sd}" => Signature::Dict {
                key: ChildSignature::Static {
                    child: &Signature::Str
                },
                value: ChildSignature::Static {
                    child: &Signature::F64
                }
            },
            "a{yy}" => Signature::Dict {
                key: ChildSignature::Static {
                    child: &Signature::U8
                },
                value: ChildSignature::Static {
                    child: &Signature::U8
                }
            },
            "a{sv}" => Signature::Dict {
                key: ChildSignature::Static {
                    child: &Signature::Str,
                },
                value: ChildSignature::Static {
                    child: &Signature::Value
                }
            },
            "a{sa{sv}}" => Signature::Dict {
                key: ChildSignature::Static {
                    child: &Signature::Str
                },
                value: ChildSignature::Static {
                    child: &Signature::Dict {
                        key: ChildSignature::Static {
                            child: &Signature::Str,
                        },
                        value: ChildSignature::Static {
                            child: &Signature::Value
                        }
                    }
                }
            },
            "a{sa(ux)}" => Signature::Dict {
                key: ChildSignature::Static {
                    child: &Signature::Str
                },
                value: ChildSignature::Static {
                    child: &Signature::Array(ChildSignature::Static {
                    child: &Signature::Structure(FieldsSignatures::Static {
                        fields: &[&Signature::U32, &Signature::I64]
                    })}),
                }
            },
            "(x)" => Signature::Structure(FieldsSignatures::Static {
                fields: &[&Signature::I64]
            }),
            "(x(isy))" => Signature::Structure(FieldsSignatures::Static {
                fields: &[
                    &Signature::I64,
                    &Signature::Structure(FieldsSignatures::Static {
                        fields: &[&Signature::I32, &Signature::Str, &Signature::U8]
                    }),
                ]
            }),
            "(xa(isy))" => Signature::Structure(FieldsSignatures::Static {
                fields: &[
                    &Signature::I64,
                    &Signature::Array(ChildSignature::Static {
                        child: &Signature::Structure(FieldsSignatures::Static {
                            fields: &[&Signature::I32, &Signature::Str, &Signature::U8]
                        }),
                    }),
                ]
            }),
            "(xa(s))" => Signature::Structure(FieldsSignatures::Static {
                fields: &[
                    &Signature::I64,
                    &Signature::Array(ChildSignature::Static {
                        child: &Signature::Structure(FieldsSignatures::Static {
                            fields: &[&Signature::Str]
                        }),
                    }),
                ]
            })
        );
        validate!("h" => Signature::Fd);
    }

    macro_rules! invalidate {
        ($($signature:literal),+) => {
            $(
                assert!(validate($signature).is_err());
            )+
        };
    }

    #[test]
    fn invalid_strings() {
        invalidate!(
            "a",
            "a{}",
            "a{y",
            "a{y}",
            "a{y}a{y}",
            "a{y}a{y}a{y}",
            "z",
            "()",
            "(x",
            "(x())",
            "(xa()",
            "(xa(s)",
            "(xs",
            "xs)",
            "s/",
            "a{yz}"
        );
    }
}
