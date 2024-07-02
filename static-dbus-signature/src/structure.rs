use crate::{dynamic_type::DynamicType, r#type::Type, signature::Signature};

pub struct Structure<'s> {
    fields: Vec<Signature<'s>>,
}

impl Structure<'_> {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn field<T: Type + ?Sized>(mut self) -> Self {
        self.fields.push(T::SIGNATURE);

        self
    }
}

impl DynamicType for Structure<'_> {
    fn signature(&self) -> Signature<'_> {
        Signature::Structure {
            fields: &self.fields,
        }
    }
}
