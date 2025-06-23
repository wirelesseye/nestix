use std::{any::Any, cell::OnceCell, fmt::Debug, marker::PhantomData, rc::Rc};

use bon::Builder;

use crate::{
    components::{component_id, Component, ComponentID},
    props::Props,
};

pub struct HandleValue<T> {
    rc: Rc<OnceCell<Box<dyn Any>>>,
    phantom: PhantomData<T>,
}

impl<T> Clone for HandleValue<T> {
    fn clone(&self) -> Self {
        Self {
            rc: self.rc.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<T: 'static> HandleValue<T> {
    pub(crate) fn from_rc(rc: Rc<OnceCell<Box<dyn Any>>>) -> Self {
        Self {
            rc,
            phantom: PhantomData,
        }
    }

    pub fn set(&self, value: T) {
        let _ = self.rc.set(Box::new(value));
    }

    pub fn get(&self) -> &T {
        self.rc.get().unwrap().downcast_ref().unwrap()
    }
}

#[derive(Clone)]
pub enum Handle {
    Value(Rc<OnceCell<Box<dyn Any>>>),
    Callback(Rc<dyn Fn(Box<dyn Any>)>),
}

impl Handle {
    pub fn provide(&self, value: Box<dyn Any>) {
        match self {
            Handle::Value(rc) => {
                let _ = rc.set(value);
            }
            Handle::Callback(cb) => cb(value),
        }
    }
}

impl Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(arg0) => f.debug_tuple("Value").field(arg0).finish(),
            Self::Callback(_) => f.debug_tuple("Callback").finish(),
        }
    }
}

impl<T> From<HandleValue<T>> for Handle {
    fn from(value: HandleValue<T>) -> Self {
        Self::Value(value.rc)
    }
}

impl From<Rc<dyn Fn(Box<dyn Any>)>> for Handle {
    fn from(value: Rc<dyn Fn(Box<dyn Any>)>) -> Self {
        Self::Callback(value)
    }
}

#[derive(Debug, Builder, Clone)]
pub struct ElementOptions {
    #[builder(into)]
    pub key: Option<String>,
    #[builder(into)]
    pub handle: Option<Handle>,
}

#[derive(Debug, Clone)]
pub struct Element {
    pub(crate) component_id: ComponentID,
    pub(crate) props: Rc<dyn Props>,
    pub(crate) options: Rc<ElementOptions>,
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.component_id == other.component_id && !self.props.has_changed(&*other.props)
    }
}

impl Element {
    #[inline]
    pub fn component_id(&self) -> ComponentID {
        self.component_id
    }

    #[inline]
    pub fn props(&self) -> &dyn Props {
        self.props.as_ref()
    }

    #[inline]
    pub fn options(&self) -> &ElementOptions {
        &self.options
    }

    #[inline]
    pub fn set_options(&mut self, options: impl Into<Rc<ElementOptions>>) {
        self.options = options.into();
    }
}

pub fn create_element<C: Component>(props: C::Props, options: ElementOptions) -> Element {
    Element {
        component_id: component_id::<C>(),
        props: Rc::new(props),
        options: Rc::new(options),
    }
}
