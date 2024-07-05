use crate::signature::{Signature, StructSignature};

pub trait Type {
    const SIGNATURE: Signature;
}

impl<T> Type for &T
where
    T: Type + ?Sized,
{
    const SIGNATURE: Signature = T::SIGNATURE;
}

impl<T: Type> Type for [T] {
    const SIGNATURE: Signature = Signature::Array {
        child: &T::SIGNATURE,
    };
}

impl<T: Type> Type for (T,) {
    const SIGNATURE: Signature = Signature::Structure(StructSignature::Static(&[&T::SIGNATURE]));
}
// TODO: Use a macro for for generating all tuple impls

impl Type for i32 {
    const SIGNATURE: Signature = Signature::I32;
}
