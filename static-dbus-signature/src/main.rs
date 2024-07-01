#[derive(Debug, Clone, Copy)]
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

    // Container types
    Array {
        child: &'static Signature,
    },
    Dict {
        key: &'static Signature,
        value: &'static Signature,
    },
    Structure {
        fields: &'static [Signature],
    },
    #[cfg(feature = "gvariant")]
    Maybe {
        child: &'static Signature,
    },

    #[cfg(unix)]
    Fd,
}

fn main() {}
