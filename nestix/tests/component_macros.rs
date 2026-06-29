use std::{cell::Cell, rc::Rc};

use nestix::{Element, Fragment, Layout, build_props, component, layout, mount_root, props};

#[props]
struct CounterProps {
    count: Rc<Cell<usize>>,
}

#[component]
fn Counter(props: &CounterProps) {
    let count = props.count.get();
    count.set(count.get() + 1);
}

#[props]
struct WrapperProps {
    count: Rc<Cell<usize>>,
}

#[component]
fn Wrapper(props: &WrapperProps) -> Element {
    layout! {
        Fragment {
            Counter(.count = props.count.clone())
        }
    }
}

#[props]
struct DefaultChildrenProps {
    #[props(default)]
    children: Layout,
}

#[component]
fn DefaultChildren(props: &DefaultChildrenProps) -> Element {
    layout! {
        Fragment {
            $(props.children.clone())
        }
    }
}

#[test]
fn generated_props_and_component_can_be_mounted_directly() {
    let count = Rc::new(Cell::new(0));
    let element = nestix::create_element::<Counter>(build_props!(CounterProps(
        .count = count.clone(),
    )));

    mount_root(&element);

    assert_eq!(count.get(), 1);
}

#[test]
fn layout_macro_mounts_nested_components_through_fragment() {
    let count = Rc::new(Cell::new(0));
    let element = layout! {
        Wrapper(.count = count.clone())
    };

    mount_root(&element);

    assert_eq!(count.get(), 1);
}

#[test]
fn generated_default_layout_props_start_empty() {
    let element = layout! {
        DefaultChildren()
    };

    mount_root(&element);
}
