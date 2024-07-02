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

pub trait Type {
    const SIGNATURE: Signature<'static>;
}

impl<T> Type for &T
where
    T: Type + ?Sized,
{
    const SIGNATURE: Signature<'static> = T::SIGNATURE;
}

impl<T: Type> Type for [T] {
    const SIGNATURE: Signature<'static> = Signature::Array {
        child: &T::SIGNATURE,
    };
}

impl<T: Type> Type for (T,) {
    const SIGNATURE: Signature<'static> = Signature::Structure {
        fields: &[T::SIGNATURE],
    };
}
// TODO: Use a macro for for generating all tuple impls

impl Type for i32 {
    const SIGNATURE: Signature<'static> = Signature::I32;
}

pub trait DynamicType {
    fn signature(&self) -> Signature<'_>;
}

struct Structure<'s> {
    fields: Vec<Signature<'s>>,
}

impl Structure<'_> {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn field<T: Type + ?Sized>(mut self) -> Self {
        self.fields.push(T::SIGNATURE);

        self
    }
}

impl DynamicType for Structure<'_> {
    fn signature(&self) -> Signature<'_> {
        Signature::Structure {
            fields: &self.fields,
        }
    }
}

fn main() {
    let sig = Structure::new().field::<i32>().field::<&[&[i32]]>();

    //println!("{}", sig.as_str());
}
