use std::sync::Arc;

use crate::{
    dynamic_type::DynamicType,
    r#type::Type,
    signature::{FieldsSignatures, Signature},
};

#[derive(Debug, Clone)]
pub struct Structure {
    fields: Arc<[Signature]>,
}

impl Structure {
    // These methods will be the task of the builder. Structure is immutable in zvariant and it should remain so.
    /*pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn field<T: Type + ?Sized>(mut self) -> Self {
        self.fields.push(T::SIGNATURE.clone());

        self
    }*/
}

impl DynamicType for Structure {
    fn signature(&self) -> Signature {
        // NOT nice to have to clone here. :(
        Signature::Structure(FieldsSignatures::Dynamic {
            fields: self.fields.clone(),
        })
    }
}
