use crate::signature::{ArraySignature, Signature, StructSignature};

pub trait Type {
    const SIGNATURE: &'static Signature;
}

impl<T> Type for &T
where
    T: Type + ?Sized,
{
    const SIGNATURE: &'static Signature = &T::SIGNATURE;
}

impl<T: Type> Type for [T] {
    const SIGNATURE: &'static Signature = &Signature::Array(ArraySignature::Static {
        child: &T::SIGNATURE,
    });
}

impl<A: Type> Type for (A,) {
    const SIGNATURE: &'static Signature = &Signature::Structure(StructSignature::Static {
        fields: &[A::SIGNATURE],
    });
}
impl<A: Type, B: Type> Type for (A, B) {
    const SIGNATURE: &'static Signature = &Signature::Structure(StructSignature::Static {
        fields: &[A::SIGNATURE, B::SIGNATURE],
    });
}
impl<A: Type, B: Type, C: Type> Type for (A, B, C) {
    const SIGNATURE: &'static Signature = &Signature::Structure(StructSignature::Static {
        fields: &[A::SIGNATURE, B::SIGNATURE, C::SIGNATURE],
    });
}
impl<A: Type, B: Type, C: Type, D: Type> Type for (A, B, C, D) {
    const SIGNATURE: &'static Signature = &Signature::Structure(StructSignature::Static {
        fields: &[A::SIGNATURE, B::SIGNATURE, C::SIGNATURE, D::SIGNATURE],
    });
}
// TODO: Use a macro for for generating all tuple impls

impl Type for i32 {
    const SIGNATURE: &'static Signature = &Signature::I32;
}
impl Type for &str {
    const SIGNATURE: &'static Signature = &Signature::Str;
}
impl Type for bool {
    const SIGNATURE: &'static Signature = &Signature::Bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature() {
        // i32
        assert_eq!(i32::SIGNATURE, &Signature::I32);
        assert_eq!(i32::SIGNATURE.to_string(), "i");

        // Array of i32
        let sig = <&[i32]>::SIGNATURE;
        assert_eq!(
            sig,
            &Signature::Array(ArraySignature::Static {
                child: &Signature::I32
            })
        );
        assert_eq!(sig.to_string(), "ai");

        // Structure of (i32, &str, &[&[i32]], bool)
        let sig = <(i32, &str, &[&[i32]], bool)>::SIGNATURE;
        assert_eq!(
            sig,
            &Signature::Structure(StructSignature::Dynamic {
                fields: vec![
                    Signature::I32,
                    Signature::Str,
                    Signature::Array(ArraySignature::Dynamic {
                        child: Box::new(Signature::Array(ArraySignature::Static {
                            child: &Signature::I32,
                        })),
                    }),
                    Signature::Bool
                ]
            })
        );
        assert_eq!(sig.to_string(), "(isaaib)");
    }
}
