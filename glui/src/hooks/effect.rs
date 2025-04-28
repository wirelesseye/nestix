use std::any::Any;

use crate::current_app_model;

pub fn effect_cleanup<D: 'static + PartialEq + Eq, C: 'static + Fn() -> ()>(
    dependency: D,
    setup: impl FnOnce(&D) -> C,
) {
    let app_model = current_app_model().unwrap();
    if let Some(rc) = app_model.get_value() {
        let store = rc.downcast_ref::<EffectStore>().unwrap();
        let memo_dependency = store.dependency.downcast_ref::<D>().unwrap();
        if dependency != *memo_dependency {
            if let Some(cleanup) = &store.cleanup_fn {
                cleanup();
            }
            let cleanup_fn = setup(&dependency);
            app_model.backward_value();
            app_model.set_value(EffectStore {
                dependency: Box::new(dependency),
                cleanup_fn: Some(Box::new(cleanup_fn)),
            });
        }
    } else {
        let cleanup_fn = setup(&dependency);
        app_model.set_value(EffectStore {
            dependency: Box::new(dependency),
            cleanup_fn: Some(Box::new(cleanup_fn)),
        });
    }
}

pub fn effect<D: 'static + PartialEq + Eq>(dependency: D, setup: impl FnOnce(&D) -> ()) {
    let app_model = current_app_model().unwrap();
    if let Some(rc) = app_model.get_value() {
        let store = rc.downcast_ref::<EffectStore>().unwrap();
        let memo_dependency = store.dependency.downcast_ref::<D>().unwrap();
        if dependency != *memo_dependency {
            setup(&dependency);
            app_model.backward_value();
            app_model.set_value(EffectStore {
                dependency: Box::new(dependency),
                cleanup_fn: None,
            });
        }
    } else {
        setup(&dependency);
        app_model.set_value(EffectStore {
            dependency: Box::new(dependency),
            cleanup_fn: None,
        });
    }
}

pub(crate) struct EffectStore {
    dependency: Box<dyn Any>,
    cleanup_fn: Option<Box<dyn Fn() -> ()>>,
}

impl Drop for EffectStore {
    fn drop(&mut self) {
        if let Some(cleanup) = &self.cleanup_fn {
            cleanup();
        }
    }
}
