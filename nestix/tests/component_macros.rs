use std::{cell::Cell, rc::Rc};

use nestix::{
    Element, Fragment, Layout, Props, build_props, component, create_state, layout, mount_root,
    props, scoped_effect,
};

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

#[props(extensible(FirstPropsExt, FirstPropsWrapper))]
struct FirstProps {
    #[props(default)]
    first: usize,
}

#[props(extensible(SecondPropsExt, SecondPropsWrapper))]
struct SecondProps {
    #[props(default)]
    second: usize,
}

#[props]
struct MultiExtendsProps {
    #[props(extends(FirstPropsExt, FirstPropsWrapper))]
    first_props: FirstProps,

    #[props(extends(SecondPropsExt, SecondPropsWrapper))]
    second_props: SecondProps,

    own: usize,
}

#[props(
    extensible(SpacingPropsExt, SpacingPropsWrapper),
    group(inset => [left, right, top, bottom]),
    group(vertical => [top, bottom]),
)]
struct SpacingProps {
    #[props(default)]
    left: usize,
    #[props(default)]
    right: usize,
    #[props(default)]
    top: usize,
    #[props(default)]
    bottom: usize,
}

#[props]
struct SpacingExtendsProps {
    #[props(extends(SpacingPropsExt, SpacingPropsWrapper))]
    spacing_props: SpacingProps,
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

struct ScopedEffectComponentProps {
    value: nestix::State<i32>,
    observed: Rc<Cell<i32>>,
}

impl Props for ScopedEffectComponentProps {}

#[component]
fn ScopedEffectComponent(props: &ScopedEffectComponentProps, element: &Element) {
    scoped_effect!(
        element,
        [props.value, props.observed] || {
            observed.set(value.get());
        }
    );
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

#[test]
fn generated_props_can_extend_multiple_prop_groups() {
    use first_props_builder::FirstPropsBuilderExtFirst;
    use second_props_builder::SecondPropsBuilderExtSecond;

    let props = build_props!(MultiExtendsProps(
        .first = 1usize,
        .second = 2usize,
        .own = 3usize,
    ));

    assert_eq!(props.first_props.first.get(), 1);
    assert_eq!(props.second_props.second.get(), 2);
    assert_eq!(props.own.get(), 3);
}

#[test]
fn generated_props_can_set_grouped_fields() {
    let props = build_props!(SpacingProps(
        .vertical = 8usize,
    ));

    assert_eq!(props.left.get(), 0);
    assert_eq!(props.right.get(), 0);
    assert_eq!(props.top.get(), 8);
    assert_eq!(props.bottom.get(), 8);
}

#[test]
fn generated_props_can_set_grouped_fields_through_extends() {
    use spacing_props_builder::SpacingPropsBuilderExtInset;

    let props = build_props!(SpacingExtendsProps(
        .inset = 6usize,
    ));

    assert_eq!(props.spacing_props.left.get(), 6);
    assert_eq!(props.spacing_props.right.get(), 6);
    assert_eq!(props.spacing_props.top.get(), 6);
    assert_eq!(props.spacing_props.bottom.get(), 6);
}

#[test]
fn scoped_effect_macro_cancels_effect_on_unmount() {
    let value = create_state(1);
    let observed = Rc::new(Cell::new(0));
    let element = nestix::create_element::<ScopedEffectComponent>(ScopedEffectComponentProps {
        value: value.clone(),
        observed: observed.clone(),
    });

    mount_root(&element);

    assert_eq!(observed.get(), 1);

    value.set(2);
    assert_eq!(observed.get(), 2);

    element.unmount();

    value.set(3);
    assert_eq!(observed.get(), 2);
}
