#[derive(Debug, Clone, Copy)]
pub enum Signature<'s> {
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
        child: &'s Signature<'s>,
    },
    Dict {
        key: &'s Signature<'s>,
        value: &'s Signature<'s>,
    },
    Structure {
        fields: &'s [Signature<'s>],
    },
    #[cfg(feature = "gvariant")]
    Maybe {
        child: &'s Signature<'s>,
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