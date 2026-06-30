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

#[props(
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

#[props(default)]
struct ViewProps {
    #[props(default)]
    margin: f32,
}

#[props]
struct ButtonProps {
    #[props(nested)]
    view_props: ViewProps,

    #[props(default)]
    title: String,
}

#[props]
struct PositionedViewProps {
    #[props(start)]
    x: i32,

    #[props(start)]
    y: f32,

    #[props(default)]
    margin: f32,
}

#[props]
struct PositionedButtonProps {
    #[props(nested(x: i32, y: f32))]
    view_props: PositionedViewProps,
}

#[props(default)]
struct OptionalProps {
    label: Option<String>,
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
fn generated_props_can_build_nested_fields() {
    let view_props = ButtonProps::view_props_builder()
        .margin(nestix::prop_value!(2.0f32))
        .build();
    assert_eq!(view_props.margin.get(), 2.0);

    let props = build_props!(ButtonProps(
        .view_props(
            .margin = 3.0f32,
        ),
        .title = "Click".to_string(),
    ));

    assert_eq!(props.view_props.margin.get(), 3.0);
    assert_eq!(props.title.get(), "Click");

    let explicit_nested = build_props!(ViewProps(
        .margin = 5.0f32,
    ));
    let props = build_props!(ButtonProps(
        .view_props = explicit_nested,
    ));

    assert_eq!(props.view_props.margin.get(), 5.0);
    assert_eq!(props.title.get(), "");
}

#[test]
fn generated_props_can_build_nested_fields_with_start_args() {
    let view_props = PositionedButtonProps::view_props_builder(1, 2.0)
        .margin(nestix::prop_value!(3.0f32))
        .build();
    assert_eq!(view_props.x.get(), 1);
    assert_eq!(view_props.y.get(), 2.0);
    assert_eq!(view_props.margin.get(), 3.0);

    let props = build_props!(PositionedButtonProps(
        .view_props(
            4,
            5.0f32,
            .margin = 6.0f32,
        ),
    ));

    assert_eq!(props.view_props.x.get(), 4);
    assert_eq!(props.view_props.y.get(), 5.0);
    assert_eq!(props.view_props.margin.get(), 6.0);
}

#[test]
fn generated_props_can_derive_default_when_all_fields_default() {
    let view_props = ViewProps::default();
    assert_eq!(view_props.margin.get(), 0.0);

    let optional_props = OptionalProps::default();
    assert_eq!(optional_props.label.get(), None);
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
