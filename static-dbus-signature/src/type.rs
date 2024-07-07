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

impl<A: Type> Type for (A,) {
    const SIGNATURE: &'static Signature =
        &Signature::Structure(StructSignature::Static(&[A::SIGNATURE]));
}
impl<A: Type, B: Type> Type for (A, B) {
    const SIGNATURE: &'static Signature =
        &Signature::Structure(StructSignature::Static(&[A::SIGNATURE, B::SIGNATURE]));
}
impl<A: Type, B: Type, C: Type> Type for (A, B, C) {
    const SIGNATURE: &'static Signature = &Signature::Structure(StructSignature::Static(&[
        A::SIGNATURE,
        B::SIGNATURE,
        C::SIGNATURE,
    ]));
}
impl<A: Type, B: Type, C: Type, D: Type> Type for (A, B, C, D) {
    const SIGNATURE: &'static Signature = &Signature::Structure(StructSignature::Static(&[
        A::SIGNATURE,
        B::SIGNATURE,
        C::SIGNATURE,
        D::SIGNATURE,
    ]));
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
