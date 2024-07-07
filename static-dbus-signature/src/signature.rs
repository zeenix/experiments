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

/*impl Signature {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Signature::U8 => "y",
            Signature::Bool => "b",
            Signature::I16 => "n",
            Signature::U16 => "q",
            Signature::I32 => "i",
            Signature::U32 => "u",
            Signature::I64 => "x",
            Signature::U64 => "t",
            Signature::F64 => "d",
            Signature::Str => "s",
            Signature::Signature => "g",
            Signature::ObjectPath => "o",
            Signature::Value => "v",
            #[cfg(unix)]
            Signature::Fd => "h",
            Signature::Array { child } => {
                concat_const::concat!("a", child.as_str())
            }
            Signature::Dict { key, value } => {
                concat_const::concat!("a{", key.as_str(), value.as_str(), "}")
            }
            Signature::Structure { fields } => {
                let fields = fields
                    .iter()
                    .map(|f| f.as_str())
                    .collect::<[]>()
                    .join("");
                &format!("({})", fields)
            }
            #[cfg(feature = "gvariant")]
            Signature::Maybe { child } => {
                let child = child.as_str();
                concat_const::concat!("m{}", child)
            }
        }
    }
}*/

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StructSignature {
    Static(&'static [&'static Signature]),
    Dynamic(Vec<Signature>),
}
