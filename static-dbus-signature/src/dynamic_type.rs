use crate::signature::Signature;

pub trait DynamicType {
    fn signature(&self) -> Signature<'_>;
}
