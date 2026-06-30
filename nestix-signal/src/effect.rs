use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    panic::Location,
    rc::Rc,
};

use crate::{get_config, shared::Shared};

thread_local! {
    static CURRENT_EFFECT: RefCell<Option<Shared<Effect>>> = RefCell::new(None);
    static RUNNING_EFFECTS: RefCell<HashSet<Shared<Effect>>> = RefCell::new(HashSet::new());
    static BATCH_DEPTH: Cell<usize> = const { Cell::new(0) };
    static PENDING_EFFECTS: RefCell<Vec<(Shared<Effect>, &'static Location<'static>)>> = const { RefCell::new(Vec::new()) };
    static PENDING_EFFECT_SET: RefCell<HashSet<Shared<Effect>>> = RefCell::new(HashSet::new());
}

pub(crate) fn current_effect() -> Option<Shared<Effect>> {
    CURRENT_EFFECT.with_borrow(|effect| {
        effect
            .as_ref()
            .filter(|effect| !effect.is_cancelled())
            .cloned()
    })
}

pub(crate) fn set_current_effect(effect: Option<Shared<Effect>>) {
    CURRENT_EFFECT.replace(effect);
}

pub(crate) fn is_effect_running(effect: &Shared<Effect>) -> bool {
    RUNNING_EFFECTS.with_borrow(|effects| effects.contains(effect))
}

pub(crate) fn start_effect(effect: Shared<Effect>) {
    RUNNING_EFFECTS.with_borrow_mut(|effects| effects.insert(effect));
}

pub(crate) fn end_effect(effect: &Shared<Effect>) {
    RUNNING_EFFECTS.with_borrow_mut(|effects| effects.remove(effect));
}

fn is_batching() -> bool {
    BATCH_DEPTH.with(|depth| depth.get() > 0)
}

pub(crate) struct Effect {
    location: &'static Location<'static>,
    callback: Shared<dyn Fn()>,
    dependency_sets: RefCell<HashSet<Shared<RefCell<HashSet<Shared<Effect>>>>>>,
    cancelled: Cell<bool>,
    batched: bool,
}

impl Effect {
    pub fn new(location: &'static Location, callback: Shared<dyn Fn()>) -> Shared<Self> {
        Self::new_with_batching(location, callback, true)
    }

    pub fn new_unbatched(location: &'static Location, callback: Shared<dyn Fn()>) -> Shared<Self> {
        Self::new_with_batching(location, callback, false)
    }

    fn new_with_batching(
        location: &'static Location,
        callback: Shared<dyn Fn()>,
        batched: bool,
    ) -> Shared<Self> {
        Shared::new(Effect {
            location,
            callback,
            dependency_sets: RefCell::new(HashSet::new()),
            cancelled: Cell::new(false),
            batched,
        })
    }

    pub fn add_dependency_set(&self, dependency_set: Shared<RefCell<HashSet<Shared<Effect>>>>) {
        if !self.is_cancelled() {
            self.dependency_sets.borrow_mut().insert(dependency_set);
        }
    }

    pub fn take_dependency_sets(&self) -> HashSet<Shared<RefCell<HashSet<Shared<Effect>>>>> {
        self.dependency_sets.take()
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.get()
    }

    fn should_batch(&self) -> bool {
        self.batched
    }

    fn cancel(&self, effect: &Shared<Effect>) {
        self.cancelled.set(true);
        for dependency_set in self.take_dependency_sets() {
            dependency_set.borrow_mut().remove(effect);
        }
        end_effect(effect);
    }
}

/// A handle that can cancel a registered effect.
///
/// Dropping the handle does not cancel the effect. Call [`EffectHandle::cancel`]
/// when the effect should stop rerunning and unsubscribe from its dependencies.
#[derive(Clone)]
pub struct EffectHandle {
    effect: Shared<Effect>,
}

impl EffectHandle {
    fn new(effect: Shared<Effect>) -> Self {
        Self { effect }
    }

    /// Cancels this effect and removes it from all currently tracked
    /// dependency sets.
    ///
    /// Calling `cancel` more than once is harmless.
    pub fn cancel(&self) {
        self.effect.cancel(&self.effect);
    }

    /// Returns whether this effect has been canceled.
    pub fn is_cancelled(&self) -> bool {
        self.effect.is_cancelled()
    }
}

/// Registers a reactive side effect and runs it immediately.
///
/// Signals read while `f` runs become dependencies. When any of those signals
/// changes, the effect runs again and refreshes its dependency list.
#[track_caller]
pub fn effect(f: impl Fn() + 'static) -> EffectHandle {
    let location = Location::caller();
    let callback = Shared::from(Rc::new(f) as Rc<dyn Fn()>);
    let effect = Effect::new(location, callback);
    run_effect(&effect, location);
    EffectHandle::new(effect)
}

struct BatchGuard;

impl Drop for BatchGuard {
    fn drop(&mut self) {
        let is_outermost = BATCH_DEPTH.with(|depth| {
            let next = depth.get() - 1;
            depth.set(next);
            next == 0
        });

        if is_outermost {
            if std::thread::panicking() {
                clear_pending_effects();
            } else {
                flush_pending_effects();
            }
        }
    }
}

/// Runs `f` while grouping reactive updates.
///
/// Effects caused by state changes inside `f` are queued and run once after the
/// outermost batch completes. Computed values are still invalidated
/// immediately, so reads inside the batch observe current derived values.
pub fn batch<T>(f: impl FnOnce() -> T) -> T {
    BATCH_DEPTH.with(|depth| depth.set(depth.get() + 1));

    let guard = BatchGuard;
    let value = f();
    drop(guard);
    value
}

fn flush_pending_effects() {
    let pending = PENDING_EFFECTS.with_borrow_mut(std::mem::take);
    PENDING_EFFECT_SET.with_borrow_mut(|effects| effects.clear());

    for (effect, location) in pending {
        run_effect(&effect, location);
    }
}

fn clear_pending_effects() {
    PENDING_EFFECTS.with_borrow_mut(|effects| effects.clear());
    PENDING_EFFECT_SET.with_borrow_mut(|effects| effects.clear());
}

pub(crate) fn notify_effect(effect: &Shared<Effect>, location: &'static Location<'static>) {
    if is_batching() && effect.should_batch() {
        let inserted = PENDING_EFFECT_SET.with_borrow_mut(|effects| effects.insert(effect.clone()));
        if inserted {
            PENDING_EFFECTS.with_borrow_mut(|effects| effects.push((effect.clone(), location)));
        }
    } else {
        run_effect(effect, location);
    }
}

pub(crate) fn run_effect(effect: &Shared<Effect>, location: &'static Location<'static>) {
    if effect.is_cancelled() {
        return;
    }

    #[cfg(debug_assertions)]
    {
        let config = get_config();
        if config.detect_cyclic && is_effect_running(effect) {
            log::warn!(
                "cyclic update detected\n\tat {}:{}\nwhen trying to modify value\n\tat {}:{}",
                effect.location.file(),
                effect.location.line(),
                location.file(),
                location.line(),
            );
        }
    }

    // Cleanup old dependencies
    for dependency_set in effect.dependency_sets.take() {
        dependency_set.borrow_mut().remove(effect);
    }

    // Execute effect
    start_effect(effect.clone());
    let prev = current_effect();
    set_current_effect(Some(effect.clone()));
    (effect.callback)();
    set_current_effect(prev);
    end_effect(effect);
}
