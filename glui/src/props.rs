use std::{any::Any, fmt::Debug};

#[allow(private_bounds)]
pub trait Props: AsAny {
    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Props(..)")
    }

    fn has_changed(&self, prev: &dyn Props) -> bool;
}

impl Props for () {
    fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Props").field(&()).finish()
    }

    fn has_changed(&self, prev: &dyn Props) -> bool {
        if let Some(_) = prev.as_any().downcast_ref::<()>() {
            false
        } else {
            true
        }
    }
}

impl Debug for dyn Props {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.debug_fmt(f)
    }
}

impl dyn Props {
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}

pub(crate) trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
