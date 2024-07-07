use crate::{
    dynamic_type::DynamicType,
    r#type::Type,
    signature::{Signature, StructSignature},
};

pub struct Structure {
    fields: Vec<Signature>,
}

impl Structure {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn field<T: Type + ?Sized>(mut self) -> Self {
        self.fields.push(T::SIGNATURE.clone());

        self
    }
}

impl DynamicType for Structure {
    fn signature(&self) -> Signature {
        // NOT nice to have to clone here. :(
        Signature::Structure(StructSignature::Dynamic {
            fields: self.fields.clone(),
        })
    }
}
