use std::{any::Any, cell::OnceCell, fmt::Debug, marker::PhantomData, rc::Rc};

use crate::{
    components::{component_id, Component, ComponentID},
    props::Props,
};

#[derive(Debug)]
pub struct ValueReceiver<Handle> {
    rc: Rc<OnceCell<Handle>>,
    phantom: PhantomData<Handle>,
}

impl<Handle> Clone for ValueReceiver<Handle> {
    fn clone(&self) -> Self {
        Self {
            rc: self.rc.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<Handle: 'static> ValueReceiver<Handle> {
    pub(crate) fn from_rc(rc: Rc<OnceCell<Handle>>) -> Self {
        Self {
            rc,
            phantom: PhantomData,
        }
    }

    pub fn set(&self, value: Handle) {
        let _ = self.rc.set(value);
    }

    pub fn get(&self) -> Option<&Handle> {
        self.rc.get()
    }
}

pub enum Receiver<Handle> {
    Value(ValueReceiver<Handle>),
    Callback(Rc<dyn Fn(Handle)>),
}

impl<Handle> Clone for Receiver<Handle> {
    fn clone(&self) -> Self {
        match self {
            Self::Value(arg0) => Self::Value(arg0.clone()),
            Self::Callback(arg0) => Self::Callback(arg0.clone()),
        }
    }
}

impl<Handle: 'static> Receiver<Handle> {
    pub fn provide(&self, value: Handle) {
        match self {
            Receiver::Value(rc) => {
                let _ = rc.set(value);
            }
            Receiver::Callback(cb) => cb(value),
        }
    }
}

impl<Handle: Debug> Debug for Receiver<Handle> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(arg0) => f.debug_tuple("Value").field(arg0).finish(),
            Self::Callback(_) => f.debug_tuple("Callback").finish(),
        }
    }
}

impl<Handle> From<ValueReceiver<Handle>> for Receiver<Handle> {
    fn from(value: ValueReceiver<Handle>) -> Self {
        Self::Value(value)
    }
}

impl<Handle> From<Rc<dyn Fn(Handle)>> for Receiver<Handle> {
    fn from(value: Rc<dyn Fn(Handle)>) -> Self {
        Self::Callback(value)
    }
}

#[derive(Clone, Debug)]
pub struct Element {
    pub(crate) component_id: ComponentID,
    pub(crate) props: Rc<dyn Props>,
    pub(crate) key: Option<Rc<String>>,
    pub(crate) receiver: Option<Rc<dyn Any>>,
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
    pub fn key(&self) -> Option<&str> {
        self.key.as_ref().map(|x| x.as_str())
    }

    #[inline]
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(Rc::new(key.into()));
        self
    }

    #[inline]
    pub fn with_maybe_key(self, key: Option<impl Into<String>>) -> Self {
        if let Some(key) = key {
            self.with_key(key)
        } else {
            self
        }
    }

    #[inline]
    pub fn receiver<Handle: 'static>(&self) -> Option<&Receiver<Handle>> {
        self.receiver
            .as_ref()
            .and_then(|receiver| receiver.downcast_ref::<Receiver<Handle>>())
    }

    #[inline]
    pub fn with_receiver<Handle: 'static>(
        mut self,
        receiver: impl Into<Receiver<Handle>>,
    ) -> Self {
        self.receiver = Some(Rc::new(receiver.into()));
        self
    }

    #[inline]
    pub fn with_maybe_receiver<Handle: 'static>(
        self,
        receiver: Option<impl Into<Receiver<Handle>>>,
    ) -> Self {
        if let Some(receiver) = receiver {
            self.with_receiver(receiver)
        } else {
            self
        }
    }
}

pub fn create_element<C: Component>(props: C::Props) -> Element {
    Element {
        component_id: component_id::<C>(),
        props: Rc::new(props),
        key: None,
        receiver: None,
    }
}
