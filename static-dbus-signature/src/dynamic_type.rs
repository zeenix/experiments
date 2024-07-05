use crate::{r#type::Type, signature::Signature};

pub trait DynamicType {
    fn signature(&self) -> Signature;
}

impl<T> DynamicType for T
where
    T: Type,
{
    fn signature(&self) -> Signature {
        T::SIGNATURE.clone()
    }
}
