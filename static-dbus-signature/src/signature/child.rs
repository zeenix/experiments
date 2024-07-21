use std::ops::Deref;
use std::sync::Arc;

use super::Signature;

#[derive(Debug, Clone)]
pub enum ChildSignature {
    Static { child: &'static Signature },
    Dynamic { child: Arc<Signature> },
}

impl Deref for ChildSignature {
    type Target = Signature;

    fn deref(&self) -> &Self::Target {
        match self {
            ChildSignature::Static { child } => child,
            ChildSignature::Dynamic { child } => child,
        }
    }
}

impl From<Arc<Signature>> for ChildSignature {
    fn from(child: Arc<Signature>) -> Self {
        ChildSignature::Dynamic { child }
    }
}

impl From<Signature> for ChildSignature {
    fn from(child: Signature) -> Self {
        ChildSignature::Dynamic {
            child: Arc::new(child),
        }
    }
}

impl From<&'static Signature> for ChildSignature {
    fn from(child: &'static Signature) -> Self {
        ChildSignature::Static { child }
    }
}
