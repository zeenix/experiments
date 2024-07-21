use std::ops::Deref;
use std::sync::Arc;

use super::Signature;

#[derive(Debug, Clone)]
pub enum FieldsSignatures {
    Static {
        fields: &'static [&'static Signature],
    },
    Dynamic {
        fields: Arc<[Signature]>,
    },
}

impl FieldsSignatures {
    pub fn iter(&self) -> impl Iterator<Item = &Signature> {
        use std::slice::Iter;

        enum Fields<'a> {
            Static(Iter<'static, &'static Signature>),
            Dynamic(Iter<'a, Signature>),
        }

        impl<'a> Iterator for Fields<'a> {
            type Item = &'a Signature;

            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Fields::Static(iter) => iter.next().map(|&f| f),
                    Fields::Dynamic(iter) => iter.next(),
                }
            }
        }

        match self {
            FieldsSignatures::Static { fields } => Fields::Static(fields.iter()),
            FieldsSignatures::Dynamic { fields } => Fields::Dynamic(fields.iter()),
        }
    }
}

impl From<Arc<[Signature]>> for FieldsSignatures {
    fn from(fields: Arc<[Signature]>) -> Self {
        FieldsSignatures::Dynamic { fields }
    }
}

impl From<Vec<Signature>> for FieldsSignatures {
    fn from(fields: Vec<Signature>) -> Self {
        FieldsSignatures::Dynamic {
            fields: fields.into(),
        }
    }
}

impl From<&'static [&'static Signature]> for FieldsSignatures {
    fn from(fields: &'static [&'static Signature]) -> Self {
        FieldsSignatures::Static { fields }
    }
}
