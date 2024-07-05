use crate::signature::{Signature, StructSignature};

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
    const SIGNATURE: &'static Signature = &Signature::Array {
        child: &T::SIGNATURE,
    };
}

impl<T: Type> Type for (T,) {
    const SIGNATURE: &'static Signature =
        &Signature::Structure(StructSignature::Static(&[T::SIGNATURE]));
}
// TODO: Use a macro for for generating all tuple impls

impl Type for i32 {
    const SIGNATURE: &'static Signature = &Signature::I32;
}
