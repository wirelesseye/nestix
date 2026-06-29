use std::{cell::Cell, rc::Rc};

use nestix_signal::{Readonly, Signal, computed, create_state, effect, untrack};

#[test]
fn state_notifies_effects_when_value_changes() {
    let count = create_state(1);
    let observed = Rc::new(Cell::new(0));

    effect({
        let count = count.clone();
        let observed = observed.clone();
        move || observed.set(count.get())
    });

    assert_eq!(observed.get(), 1);

    count.set(2);

    assert_eq!(observed.get(), 2);
}

#[test]
fn state_set_skips_equal_values_but_set_unchecked_notifies() {
    let count = create_state(1);
    let runs = Rc::new(Cell::new(0));

    effect({
        let count = count.clone();
        let runs = runs.clone();
        move || {
            count.get();
            runs.set(runs.get() + 1);
        }
    });

    assert_eq!(runs.get(), 1);

    count.set(1);
    assert_eq!(runs.get(), 1);

    count.set_unchecked(1);
    assert_eq!(runs.get(), 2);
}

#[test]
fn state_update_and_mutate_notify_dependents() {
    let values = create_state(vec![1, 2]);
    let total = Rc::new(Cell::new(0));

    effect({
        let values = values.clone();
        let total = total.clone();
        move || total.set(values.get().iter().sum())
    });

    assert_eq!(total.get(), 3);

    values.update(|values| {
        let mut next = values.clone();
        next.push(3);
        next
    });
    assert_eq!(total.get(), 6);

    values.mutate(|values| values.push(4));
    assert_eq!(total.get(), 10);
}

#[test]
fn computed_values_are_lazy_cached_and_invalidated_by_dependencies() {
    let count = create_state(2);
    let evaluations = Rc::new(Cell::new(0));

    let doubled = computed({
        let count = count.clone();
        let evaluations = evaluations.clone();
        move || {
            evaluations.set(evaluations.get() + 1);
            count.get() * 2
        }
    });

    assert_eq!(evaluations.get(), 0);

    assert_eq!(doubled.get(), 4);
    assert_eq!(doubled.get(), 4);
    assert_eq!(evaluations.get(), 1);

    count.set(3);

    assert_eq!(evaluations.get(), 1);
    assert_eq!(doubled.get(), 6);
    assert_eq!(evaluations.get(), 2);
}

#[test]
fn computed_dependencies_are_refreshed_after_each_evaluation() {
    let use_left = create_state(true);
    let left = create_state(10);
    let right = create_state(20);

    let selected = computed({
        let use_left = use_left.clone();
        let left = left.clone();
        let right = right.clone();
        move || {
            if use_left.get() {
                left.get()
            } else {
                right.get()
            }
        }
    });

    assert_eq!(selected.get(), 10);

    use_left.set(false);
    assert_eq!(selected.get(), 20);

    left.set(11);
    assert_eq!(selected.get(), 20);

    right.set(21);
    assert_eq!(selected.get(), 21);
}

#[test]
fn effects_refresh_their_dependencies_when_branches_change() {
    let use_left = create_state(true);
    let left = create_state(1);
    let right = create_state(10);
    let observed = Rc::new(Cell::new(0));
    let runs = Rc::new(Cell::new(0));

    effect({
        let use_left = use_left.clone();
        let left = left.clone();
        let right = right.clone();
        let observed = observed.clone();
        let runs = runs.clone();
        move || {
            runs.set(runs.get() + 1);
            observed.set(if use_left.get() {
                left.get()
            } else {
                right.get()
            });
        }
    });

    assert_eq!(observed.get(), 1);
    assert_eq!(runs.get(), 1);

    use_left.set(false);
    assert_eq!(observed.get(), 10);
    assert_eq!(runs.get(), 2);

    left.set(2);
    assert_eq!(observed.get(), 10);
    assert_eq!(runs.get(), 2);

    right.set(11);
    assert_eq!(observed.get(), 11);
    assert_eq!(runs.get(), 3);
}

#[test]
fn readonly_signal_tracks_wrapped_signal() {
    let source = create_state(7);
    let readonly = Readonly::new(source.clone());
    let observed = Rc::new(Cell::new(0));

    effect({
        let readonly = readonly.clone();
        let observed = observed.clone();
        move || observed.set(readonly.get())
    });

    assert_eq!(observed.get(), 7);

    source.set(8);

    assert_eq!(observed.get(), 8);
}

#[test]
fn boxed_signals_can_be_cloned_and_read() {
    let source: Box<dyn Signal<Output = i32>> = Box::new(create_state(5));
    let cloned = source.clone();

    assert_eq!(source.get(), 5);
    assert_eq!(cloned.get(), 5);
}

#[test]
fn untrack_reads_without_subscribing_the_current_effect() {
    let tracked = create_state(1);
    let ignored = create_state(10);
    let observed = Rc::new(Cell::new(0));
    let runs = Rc::new(Cell::new(0));

    effect({
        let tracked = tracked.clone();
        let ignored = ignored.clone();
        let observed = observed.clone();
        let runs = runs.clone();
        move || {
            runs.set(runs.get() + 1);
            observed.set(tracked.get() + untrack(|| ignored.get()));
        }
    });

    assert_eq!(observed.get(), 11);
    assert_eq!(runs.get(), 1);

    ignored.set(20);
    assert_eq!(observed.get(), 11);
    assert_eq!(runs.get(), 1);

    tracked.set(2);
    assert_eq!(observed.get(), 22);
    assert_eq!(runs.get(), 2);
}
