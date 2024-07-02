use crate::signature::Signature;

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
