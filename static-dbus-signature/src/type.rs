use crate::signature::{ChildSignature, FieldsSignatures, Signature};

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
    const SIGNATURE: &'static Signature = &Signature::Array(ChildSignature::Static {
        child: &T::SIGNATURE,
    });
}

macro_rules! tuple_impls {
    ($($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<$($name),+> Type for ($($name,)+)
            where
                $($name: Type,)+
            {
                const SIGNATURE: &'static Signature = &Signature::Structure(FieldsSignatures::Static {
                    fields: &[$(
                        $name::SIGNATURE,
                    )+],
                });
            }
        )+
    }
}

tuple_impls! {
    1 => (0 T0)
    2 => (0 T0 1 T1)
    3 => (0 T0 1 T1 2 T2)
    4 => (0 T0 1 T1 2 T2 3 T3)
    5 => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

impl Type for i32 {
    const SIGNATURE: &'static Signature = &Signature::I32;
}
impl Type for &str {
    const SIGNATURE: &'static Signature = &Signature::Str;
}
impl Type for bool {
    const SIGNATURE: &'static Signature = &Signature::Bool;
}

impl Type for () {
    const SIGNATURE: &'static Signature = &Signature::Unit;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_signature() {
        // Let's start with ()
        assert_eq!(<()>::SIGNATURE, &Signature::Unit);
        assert_eq!(<()>::SIGNATURE.to_string(), "");
        let sig = <()>::SIGNATURE.to_string().parse::<Signature>().unwrap();
        assert_eq!(sig, Signature::Unit);

        // i32
        assert_eq!(i32::SIGNATURE, &Signature::I32);
        assert_eq!(i32::SIGNATURE.to_string(), "i");
        let sig = i32::SIGNATURE.to_string().parse::<Signature>().unwrap();
        assert_eq!(sig, Signature::I32);

        // Array of i32
        let sig = <&[i32]>::SIGNATURE;
        assert_eq!(
            sig,
            &Signature::Array(ChildSignature::Static {
                child: &Signature::I32
            })
        );
        let sig_str = sig.to_string();
        assert_eq!(sig_str, "ai");
        let sig = sig_str.parse::<Signature>().unwrap();
        assert_eq!(
            sig,
            Signature::Array(ChildSignature::Static {
                child: &Signature::I32
            })
        );

        // Structure of (i32, &str, &[&[i32]], bool)
        let sig = <(i32, &str, &[&[i32]], bool)>::SIGNATURE;
        assert_eq!(
            sig,
            &Signature::Structure(FieldsSignatures::Dynamic {
                fields: vec![
                    Signature::I32,
                    Signature::Str,
                    Signature::Array(ChildSignature::Dynamic {
                        child: Arc::new(Signature::Array(ChildSignature::Static {
                            child: &Signature::I32,
                        })),
                    }),
                    Signature::Bool
                ]
                .into()
            })
        );
        assert_eq!(sig.to_string(), "(isaaib)");
    }
}
