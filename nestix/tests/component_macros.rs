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
    #[props(nested, default)]
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

#[props]
struct OuterProps {
    #[props(nested)]
    button_props: ButtonProps,
}

#[props(default)]
struct OptionalProps {
    label: Option<String>,
}

#[props]
struct RawProps {
    #[props(raw)]
    label: String,
}

#[props(default)]
struct DefaultRawProps {
    #[props(raw, default = "ready".to_string())]
    label: String,
}

#[props(group(labels => [primary, secondary]))]
struct RawGroupProps {
    #[props(raw)]
    primary: String,

    #[props(raw)]
    secondary: String,
}

#[component]
fn Wrapper(props: &WrapperProps) -> Element {
    layout! {
        Fragment {
            Counter(.count = props.count.clone())
        }
    }
}

#[component]
fn Button(props: &ButtonProps) {
    assert_eq!(props.title.get(), "Click");
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
fn layout_macro_accepts_direct_props_values() {
    let props = build_props!(ButtonProps(
        .title = "Click".to_string(),
    ));
    let element = layout! {
        Button$(props)
    };

    mount_root(&element);
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
    let button_builder = ButtonProps::builder().title(nestix::prop_value!("Click".to_string()));
    let view_props = button_builder
        .view_props_builder()
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
    let view_props = PositionedButtonProps::builder()
        .view_props_builder(1, 2.0)
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
fn generated_props_can_build_nested_fields_inside_nested_fields() {
    let props = build_props!(OuterProps(
        .button_props(
            .view_props(
                .margin = 7.0f32,
            ),
            .title = "Nested".to_string(),
        ),
    ));

    assert_eq!(props.button_props.view_props.margin.get(), 7.0);
    assert_eq!(props.button_props.title.get(), "Nested");
}

#[test]
fn generated_props_can_derive_default_when_all_fields_default() {
    let view_props = ViewProps::default();
    assert_eq!(view_props.margin.get(), 0.0);

    let optional_props = OptionalProps::default();
    assert_eq!(optional_props.label.get(), None);

    let raw_props = DefaultRawProps::default();
    assert_eq!(raw_props.label, "ready");
}

#[test]
fn generated_props_can_keep_raw_fields_unwrapped() {
    let props = RawProps::builder().label("plain".to_string()).build();
    assert_eq!(props.label, "plain");

    let props = build_props!(RawProps(
        .label = "from macro".to_string(),
    ));
    assert_eq!(props.label, "from macro");

    let props = build_props!(RawGroupProps(
        .labels = "shared".to_string(),
    ));
    assert_eq!(props.primary, "shared");
    assert_eq!(props.secondary, "shared");
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
