use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use nestix::{
    Component, ComponentOutput, Element, Fragment, FragmentProps, Layout, Placement, PropValue,
    Props, create_element, create_state, mount_root, scoped_effect,
};

struct Empty;

impl Component for Empty {
    type Props = ();

    fn on_mount(_: &Element) {}
}

struct CountMounts;

struct CountMountsProps {
    count: Rc<Cell<usize>>,
}

impl Props for CountMountsProps {}

impl Component for CountMounts {
    type Props = CountMountsProps;

    fn on_mount(element: &Element) {
        let count = &element
            .props()
            .downcast_ref::<CountMountsProps>()
            .unwrap()
            .count;
        count.set(count.get() + 1);
    }
}

struct Host;

impl Component for Host {
    type Props = ();

    fn on_mount(element: &Element) {
        element.provide_handle(String::from("host"));
    }
}


struct ParentWithChild;

struct ParentWithChildProps {
    child_slot: Rc<RefCell<Option<Element>>>,
}

impl Props for ParentWithChildProps {}

impl Component for ParentWithChild {
    type Props = ParentWithChildProps;

    fn on_mount(element: &Element) {
        let child = create_element::<Empty>(());
        let props = element
            .props()
            .downcast_ref::<ParentWithChildProps>()
            .unwrap();
        props.child_slot.replace(Some(child.clone()));
        child.mount(Some(element));
    }
}

#[test]
fn layout_conversions_expose_expected_elements() {
    let first = create_element::<Empty>(());
    let second = create_element::<Empty>(());

    let empty = Layout::from(());
    assert_eq!(empty.len(), 0);
    assert!(empty.get(0).is_none());
    assert_eq!(empty.iter().count(), 0);
    assert!(empty.into_elements().is_empty());

    let single = Layout::from(first.clone());
    assert_eq!(single.len(), 1);
    assert_eq!(single.get(0), Some(&first));
    assert!(single.get(1).is_none());
    assert_eq!(single[0], first);
    assert_eq!(
        single.iter().cloned().collect::<Vec<_>>(),
        vec![first.clone()]
    );
    assert_eq!(single.into_elements(), vec![first.clone()]);

    let many = Layout::from(vec![first.clone(), second.clone()]);
    assert_eq!(many.len(), 2);
    assert_eq!(many.get(0), Some(&first));
    assert_eq!(many.get(1), Some(&second));
    assert_eq!(
        many.iter().cloned().collect::<Vec<_>>(),
        vec![first, second]
    );
}

#[test]
#[should_panic(expected = "Layout index out of bounds")]
fn indexing_empty_layout_panics_with_context() {
    let empty = Layout::from(());

    let _ = &empty[0];
}

#[test]
fn prop_value_reads_plain_and_signal_values() {
    let plain = PropValue::from_plain(String::from("ready"));
    let plain_clone = plain.clone();

    assert_eq!(plain.get(), "ready");
    assert_eq!(plain_clone.get(), "ready");
    assert_eq!(plain, plain_clone);
    assert_ne!(plain, PropValue::from_plain(String::from("ready")));

    let state = create_state(1);
    let signal: PropValue<i32> = PropValue::from_signal(state.clone());
    let signal_clone = signal.clone();

    assert_eq!(signal.get(), 1);
    assert_eq!(signal_clone.get(), 1);
    assert_eq!(signal, signal_clone);

    state.set(2);

    assert_eq!(signal.get(), 2);
    assert_eq!(signal_clone.get(), 2);
}

#[test]
fn mounting_an_element_runs_lifecycle_callbacks_and_resolves_parent_handle() {
    let parent = create_element::<Host>(());
    mount_root(&parent);

    let child = create_element::<Empty>(());
    let after_mount_called = Rc::new(Cell::new(false));
    let placements = Rc::new(RefCell::new(Vec::new()));

    child.after_mount({
        let after_mount_called = after_mount_called.clone();
        move || after_mount_called.set(true)
    });
    child.on_place({
        let placements = placements.clone();
        move |placement| placements.borrow_mut().push(capture_placement(placement))
    });

    child.mount(Some(&parent));

    assert!(after_mount_called.get());
    assert_eq!(
        child.parent_handle().and_then(handle_name),
        Some(String::from("host"))
    );

    let placements = placements.borrow();
    assert_eq!(placements.len(), 1);
    assert_eq!(
        placements[0],
        CapturedPlacement {
            pred: None,
            parent: Some(String::from("host")),
            index: None,
        }
    );
}

#[test]
fn last_handle_change_callbacks_run_initially_and_follow_descendants() {
    let element = create_element::<Empty>(());
    let child = create_element::<Empty>(());
    let observed = Rc::new(RefCell::new(Vec::new()));

    element.on_last_handle_change({
        let observed = observed.clone();
        move |handle| {
            observed.borrow_mut().push(handle.and_then(handle_name));
        }
    });

    mount_root(&element);
    child.mount(Some(&element));
    child.provide_handle(String::from("first"));
    child.provide_handle(String::from("second"));
    child.unmount();

    assert_eq!(
        &*observed.borrow(),
        &[
            None,
            Some(String::from("first")),
            Some(String::from("second")),
            None,
        ]
    );
}

#[test]
fn unmount_runs_callbacks_recursively_once() {
    let child_slot = Rc::new(RefCell::new(None));
    let root = create_element::<ParentWithChild>(ParentWithChildProps {
        child_slot: child_slot.clone(),
    });
    let root_unmounts = Rc::new(Cell::new(0));

    root.on_unmount({
        let root_unmounts = root_unmounts.clone();
        move || root_unmounts.set(root_unmounts.get() + 1)
    });

    mount_root(&root);

    let child = child_slot
        .borrow()
        .clone()
        .expect("parent should mount a child");

    let child_unmounts = Rc::new(Cell::new(0));
    child.on_unmount({
        let child_unmounts = child_unmounts.clone();
        move || child_unmounts.set(child_unmounts.get() + 1)
    });

    root.unmount();
    root.unmount();

    assert_eq!(child_unmounts.get(), 1);
    assert_eq!(root_unmounts.get(), 1);
    assert!(child.parent_handle().is_none());
}

#[test]
fn scoped_effect_is_cancelled_when_element_unmounts() {
    let root = create_element::<Empty>(());
    let value = create_state(1);
    let observed = Rc::new(Cell::new(0));
    let runs = Rc::new(Cell::new(0));

    let handle = scoped_effect(&root, {
        let value = value.clone();
        let observed = observed.clone();
        let runs = runs.clone();
        move || {
            observed.set(value.get());
            runs.set(runs.get() + 1);
        }
    });

    mount_root(&root);

    assert_eq!(observed.get(), 1);
    assert_eq!(runs.get(), 1);
    assert!(!handle.is_cancelled());

    value.set(2);
    assert_eq!(observed.get(), 2);
    assert_eq!(runs.get(), 2);

    root.unmount();
    assert!(handle.is_cancelled());

    value.set(3);
    assert_eq!(observed.get(), 2);
    assert_eq!(runs.get(), 2);
}

#[test]
fn subtree_effects_are_cancelled_before_any_unmount_callback() {
    let child_slot = Rc::new(RefCell::new(None));
    let root = create_element::<ParentWithChild>(ParentWithChildProps {
        child_slot: child_slot.clone(),
    });
    mount_root(&root);
    let child = child_slot
        .borrow()
        .clone()
        .expect("parent should mount a child");

    let value = create_state(1);
    let root_runs = Rc::new(Cell::new(0));
    let child_runs = Rc::new(Cell::new(0));
    let root_handle = scoped_effect(&root, {
        let value = value.clone();
        let root_runs = root_runs.clone();
        move || {
            value.get();
            root_runs.set(root_runs.get() + 1);
        }
    });
    let child_handle = scoped_effect(&child, {
        let value = value.clone();
        let child_runs = child_runs.clone();
        move || {
            value.get();
            child_runs.set(child_runs.get() + 1);
        }
    });

    child.on_unmount({
        let value = value.clone();
        let root_handle = root_handle.clone();
        let child_handle = child_handle.clone();
        move || {
            assert!(root_handle.is_cancelled());
            assert!(child_handle.is_cancelled());
            value.set(2);
        }
    });

    root.unmount();

    assert_eq!(root_runs.get(), 1);
    assert_eq!(child_runs.get(), 1);
}

#[derive(Debug, PartialEq, Eq)]
struct CapturedPlacement {
    pred: Option<String>,
    parent: Option<String>,
    index: Option<usize>,
}

fn capture_placement(placement: &Placement) -> CapturedPlacement {
    CapturedPlacement {
        pred: placement.pred.clone().and_then(handle_name),
        parent: placement.parent.clone().and_then(handle_name),
        index: placement.index,
    }
}

fn handle_name(handle: nestix::Shared<dyn std::any::Any>) -> Option<String> {
    handle
        .downcast::<String>()
        .ok()
        .map(|value| (*value).clone())
}

#[test]
fn previous_siblings_come_from_nearest_list() {
    let parent = create_element::<Empty>(());
    let first = create_element::<Empty>(());
    let second = create_element::<Empty>(());
    let third = create_element::<Empty>(());

    mount_root(&parent);
    first.set_in_list(true);
    first.mount(Some(&parent));
    second.set_in_list(true);
    second.mount(Some(&parent));
    third.set_in_list(true);
    third.mount(Some(&parent));

    assert_eq!(first.previous_siblings(), Vec::<Element>::new());
    assert_eq!(second.previous_siblings(), vec![first.clone()]);
    assert_eq!(
        third.previous_siblings(),
        vec![second.clone(), first.clone()]
    );

    let transparent_child = create_element::<Empty>(());
    transparent_child.mount(Some(&third));

    assert_eq!(transparent_child.previous_siblings(), vec![second, first]);
}

#[test]
fn predecessor_handle_skips_logical_siblings_without_host_handles() {
    let parent = create_element::<Host>(());
    let first = create_element::<Host>(());
    let transparent = create_element::<Empty>(());
    let third = create_element::<Empty>(());
    let placements = Rc::new(RefCell::new(Vec::new()));

    third.on_place({
        let placements = placements.clone();
        move |placement| placements.borrow_mut().push(capture_placement(placement))
    });

    let fragment = create_element::<Fragment>(FragmentProps {
        children: PropValue::from_plain(Layout::from(vec![first, transparent, third.clone()])),
    });

    mount_root(&parent);
    fragment.mount(Some(&parent));

    assert_eq!(
        third.pred_handle().and_then(handle_name),
        Some(String::from("host"))
    );
    assert_eq!(
        placements.borrow().as_slice(),
        &[CapturedPlacement {
            pred: Some(String::from("host")),
            parent: Some(String::from("host")),
            index: Some(2),
        }]
    );
}

#[test]
fn fragment_notifies_later_siblings_when_previous_sibling_set_changes() {
    let first = create_element::<Empty>(());
    let second = create_element::<Empty>(());
    let third = create_element::<Empty>(());
    let third_places = Rc::new(Cell::new(0));

    third.on_place({
        let third_places = third_places.clone();
        move |_| third_places.set(third_places.get() + 1)
    });

    let children = create_state(Layout::from(vec![
        first.clone(),
        second.clone(),
        third.clone(),
    ]));
    let fragment = create_element::<Fragment>(FragmentProps {
        children: PropValue::from_signal(children.clone()),
    });

    mount_root(&fragment);

    assert_eq!(third.previous_siblings(), vec![second.clone(), first]);
    assert_eq!(third_places.get(), 1);

    children.set_unchecked(Layout::from(vec![second.clone(), third.clone()]));

    assert_eq!(third.previous_siblings(), vec![second]);
    assert_eq!(third_places.get(), 2);
}


#[test]
fn fragment_lifecycle_signal_reads_do_not_reenter_reconciliation() {
    let incidental = create_state(0);
    let first = create_element::<Empty>(());
    first.on_unmount({
        let incidental = incidental.clone();
        move || incidental.set(incidental.get() + 1)
    });

    let survivor_mounts = Rc::new(Cell::new(0));
    let survivor = create_element::<CountMounts>(CountMountsProps {
        count: survivor_mounts.clone(),
    });
    let children = create_state(Layout::from(vec![first, survivor.clone()]));
    let fragment = create_element::<Fragment>(FragmentProps {
        children: PropValue::from_signal(children.clone()),
    });
    mount_root(&fragment);

    children.set_unchecked(Layout::from(survivor));

    assert_eq!(incidental.get(), 1);
    assert_eq!(survivor_mounts.get(), 1);
}

#[test]
fn for_notifies_later_siblings_when_previous_sibling_set_changes() {
    let first = create_element::<Empty>(());
    let second = create_element::<Empty>(());
    let third = create_element::<Empty>(());
    let third_places = Rc::new(Cell::new(0));

    third.on_place({
        let third_places = third_places.clone();
        move |_| third_places.set(third_places.get() + 1)
    });

    let data = create_state(vec![1, 2, 3]);
    let list = nestix::create_for_identity_from_signal(data.clone(), {
        let first = first.clone();
        let second = second.clone();
        let third = third.clone();
        move |item| {
            PropValue::from_plain(match item.get() {
                1 => first.clone(),
                2 => second.clone(),
                3 => third.clone(),
                _ => unreachable!("test data only contains three items"),
            })
        }
    });

    mount_root(&list);

    assert_eq!(third.previous_siblings(), vec![second.clone(), first]);
    assert_eq!(third_places.get(), 1);

    data.set(vec![2, 3]);

    assert_eq!(third.previous_siblings(), vec![second]);
    assert_eq!(third_places.get(), 2);
}

#[test]
fn for_lifecycle_signal_reads_do_not_reenter_reconciliation() {
    let incidental = create_state(0);
    let first = create_element::<Empty>(());
    first.on_unmount({
        let incidental = incidental.clone();
        move || incidental.set(incidental.get() + 1)
    });

    let survivor_mounts = Rc::new(Cell::new(0));
    let survivor = create_element::<CountMounts>(CountMountsProps {
        count: survivor_mounts.clone(),
    });
    let data = create_state(vec![1, 2]);
    let list = nestix::create_for_identity_from_signal(data.clone(), {
        let first = first.clone();
        let survivor = survivor.clone();
        move |item| {
            PropValue::from_plain(match item.get() {
                1 => first.clone(),
                2 => survivor.clone(),
                _ => unreachable!("test data only contains two items"),
            })
        }
    });
    mount_root(&list);

    data.set(vec![2]);

    assert_eq!(incidental.get(), 1);
    assert_eq!(survivor_mounts.get(), 1);
}
