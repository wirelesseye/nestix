use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use nestix::{
    ContextProvider, Element, Fragment, Layout, Props, build_props, component, create_state,
    destructure, layout, mount_root, props, scoped_effect, use_context,
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

#[component(internal)]
fn InternalComponent() {}

#[test]
fn component_macro_marks_only_explicit_internal_components() {
    let public = nestix::create_element::<Counter>(build_props!(CounterProps(
        .count = Rc::new(Cell::new(0)),
    )));
    let internal = nestix::create_element::<InternalComponent>(());

    assert!(!public.is_internal());
    assert!(internal.is_internal());
    assert!(internal.component_id().is_internal());
    assert!(
        internal
            .component_id()
            .name()
            .ends_with("InternalComponent")
    );
}

#[cfg(feature = "inspector")]
#[test]
fn props_macro_generates_structured_inspection_for_plain_reactive_and_nested_fields() {
    let (title, _) = create_state("Inspector".to_string());
    let props = build_props!(ButtonProps(
        .title = title,
        .view_props(.margin = 2.5),
    ));
    let entries = nestix::InspectableProps::inspect_props(&props);

    let view = entries
        .iter()
        .find(|entry| entry.name == "view_props")
        .unwrap();
    assert_eq!(view.source, nestix::InspectPropSource::Nested);
    assert_eq!(view.children[0].name, "margin");
    assert_eq!(
        view.children[0].value,
        nestix::InspectValue::Number("2.5".into())
    );

    let title = entries.iter().find(|entry| entry.name == "title").unwrap();
    assert_eq!(title.source, nestix::InspectPropSource::Reactive);
    assert_eq!(title.value, nestix::InspectValue::Text("Inspector".into()));
    assert!(nestix::Props::as_inspectable(&props).is_some());
}

#[allow(dead_code)]
#[derive(Debug, nestix::InspectableValue)]
enum UserPosition {
    Leading,
}

#[cfg(feature = "inspector")]
fn inspect_position(_: &UserPosition) -> nestix::InspectValue {
    nestix::InspectValue::Display("custom position".into())
}

#[cfg(feature = "inspector")]
#[props]
struct InspectableValueProps {
    derived: UserPosition,
    #[props(inspect(with = inspect_position))]
    custom: UserPosition,
    #[props(inspect(skip))]
    skipped: UserPosition,
}

#[cfg(feature = "inspector")]
#[test]
fn props_inspection_supports_derived_custom_and_skipped_values() {
    let props = build_props!(InspectableValueProps(
        .derived = UserPosition::Leading,
        .custom = UserPosition::Leading,
        .skipped = UserPosition::Leading,
    ));
    let entries = nestix::InspectableProps::inspect_props(&props);

    assert_eq!(
        entries[0].value,
        nestix::InspectValue::Display("Leading".into())
    );
    assert_eq!(
        entries[1].value,
        nestix::InspectValue::Display("custom position".into())
    );
    assert_eq!(entries[2].source, nestix::InspectPropSource::Raw);
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

#[derive(Clone, PartialEq)]
struct DestructureUser {
    id: usize,
    name: String,
}

#[derive(Clone, PartialEq)]
struct DestructurePoint(i32, i32);

#[props]
struct DestructureProps {
    #[props(inspect(skip))]
    data: (String, String),
    #[props(inspect(skip))]
    user: DestructureUser,
    #[props(inspect(skip))]
    point: DestructurePoint,
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

#[props]
struct RecordingProviderProps {
    #[props(start)]
    name: &'static str,

    #[props(start)]
    mounts: Rc<RefCell<Vec<&'static str>>>,

    #[props(default)]
    children: Layout,
}

#[component]
fn RecordingProvider(props: &RecordingProviderProps) -> Element {
    props.mounts.get().borrow_mut().push(props.name.get());
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

struct HandleHost;

impl nestix::Component for HandleHost {
    type Props = ();

    fn on_mount(element: &Element) {
        element.provide_handle(String::from("host"));
    }
}

struct TransparentHost;

impl nestix::Component for TransparentHost {
    type Props = ();

    fn on_mount(element: &Element) {
        let child = nestix::create_element::<HandleHost>(());
        nestix::ComponentOutput::mount(&child, Some(element));
    }
}

impl Props for ScopedEffectComponentProps {}

#[component]
fn ScopedEffectComponent(props: &ScopedEffectComponentProps) {
    scoped_effect!(
        [props.value, props.observed] || {
            observed.set(value.get());
        }
    );
}

#[props]
struct ContextConsumerProps {
    observed: Rc<Cell<bool>>,
}

#[component]
fn ContextConsumer(props: &ContextConsumerProps) {
    props
        .observed
        .get()
        .set(use_context::<String>().as_deref().map(String::as_str) == Some("provided"));
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
fn layout_macro_accepts_if_without_else() {
    let shown_count = Rc::new(Cell::new(0));
    let hidden_count = Rc::new(Cell::new(0));

    let shown = layout! {
        Fragment {
            if true {
                Counter(.count = shown_count.clone())
            }
        }
    };
    let hidden = layout! {
        Fragment {
            if false {
                Counter(.count = hidden_count.clone())
            }
        }
    };

    mount_root(&shown);
    mount_root(&hidden);

    assert_eq!(shown_count.get(), 1);
    assert_eq!(hidden_count.get(), 0);
}

#[test]
fn layout_macro_accepts_cfg_attributes_on_children() {
    let count = Rc::new(Cell::new(0));
    let element = layout! {
        Fragment {
            #[cfg(any())]
            ComponentThatDoesNotExist
            #[cfg(all())]
            Counter(.count = count.clone())
        }
    };

    mount_root(&element);

    assert_eq!(count.get(), 1);
}

#[test]
fn layout_macro_accepts_cfg_attribute_on_its_only_element() {
    let layout = Layout::from(layout! {
        #[cfg(any())]
        ComponentThatDoesNotExist
    });

    assert_eq!(layout.len(), 0);
}

#[test]
fn layout_macro_accepts_cfg_attributes_in_reactive_branches() {
    let (show_first, _) = create_state(false);
    let count = Rc::new(Cell::new(0));
    let element = layout! {
        Fragment {
            if show_first.get() {
                TransparentHost
            } else {
                #[cfg(any())]
                FirstComponentThatDoesNotExist
                #[cfg(any())]
                SecondComponentThatDoesNotExist
                Counter(.count = count.clone())
            }
        }
    };

    mount_root(&element);

    assert_eq!(count.get(), 1);
}

#[test]
fn layout_macro_accepts_reactive_match_arms_with_dsl_bodies() {
    let (selected, set_selected) = create_state(0);
    let selected_in_layout = selected.clone();
    let first_count = Rc::new(Cell::new(0));
    let second_count = Rc::new(Cell::new(0));
    let third_count = Rc::new(Cell::new(0));
    let first_count_in_layout = first_count.clone();
    let second_count_in_layout = second_count.clone();
    let third_count_in_layout = third_count.clone();

    let element = layout! {
        Fragment {
            match selected_in_layout.get() {
                value @ (0 | 1) if value == 0 => {
                    Counter(
                        .count = if value == 0 {
                            first_count_in_layout.clone()
                        } else {
                            unreachable!()
                        },
                    )
                },
                1 => {
                    Counter(.count = second_count_in_layout.clone())
                    Counter(.count = third_count_in_layout.clone())
                },
                _ => {},
            }
        }
    };

    mount_root(&element);
    assert_eq!(first_count.get(), 1);
    assert_eq!(second_count.get(), 0);
    assert_eq!(third_count.get(), 0);

    set_selected.set(1);
    assert_eq!(first_count.get(), 1);
    assert_eq!(second_count.get(), 1);
    assert_eq!(third_count.get(), 1);

    set_selected.set(2);
    assert_eq!(first_count.get(), 1);
    assert_eq!(second_count.get(), 1);
    assert_eq!(third_count.get(), 1);
}

#[test]
fn layout_macro_match_with_one_item_per_arm_produces_a_child_layout() {
    let element = layout! {
        Fragment {
            match true {
                true => {
                    TransparentHost
                },
                false => {
                    TransparentHost
                },
            }
        }
    };

    mount_root(&element);
}

#[test]
fn layout_macro_match_reuses_binding_free_non_yielded_elements() {
    #[derive(Clone, Copy, PartialEq)]
    enum Page {
        Counter,
        TodoList,
    }

    let (page, set_page) = create_state(Page::Counter);
    let page_in_layout = page.clone();
    let matched = layout! {
        match page_in_layout.get() {
            Page::Counter => {
                TransparentHost
            },
            Page::TodoList => {
                TransparentHost
            },
        }
    };

    let first_counter = matched.get().remove(0);
    set_page.set(Page::TodoList);
    let first_todo_list = matched.get().remove(0);
    set_page.set(Page::Counter);
    let second_counter = matched.get().remove(0);
    set_page.set(Page::TodoList);
    let second_todo_list = matched.get().remove(0);

    assert_eq!(first_counter, second_counter);
    assert_eq!(first_todo_list, second_todo_list);
    assert_ne!(first_counter, first_todo_list);
}

#[test]
fn layout_macro_match_recreates_yielded_elements() {
    #[derive(Clone, Copy, PartialEq)]
    enum Page {
        Counter,
        TodoList,
    }

    let (page, set_page) = create_state(Page::Counter);
    let page_in_layout = page.clone();
    let matched = layout! {
        match page_in_layout.get() {
            Page::Counter => {
                yield TransparentHost
            },
            Page::TodoList => {
                TransparentHost
            },
        }
    };

    let first_counter = matched.get().remove(0);
    set_page.set(Page::TodoList);
    let first_todo_list = matched.get().remove(0);
    set_page.set(Page::Counter);
    let second_counter = matched.get().remove(0);
    set_page.set(Page::TodoList);
    let second_todo_list = matched.get().remove(0);

    assert_ne!(first_counter, second_counter);
    assert_eq!(first_todo_list, second_todo_list);
}

#[test]
fn layout_macro_accepts_reactive_if_directive() {
    let (show, set_show) = create_state(false);
    let show_in_layout = show.clone();
    let count = Rc::new(Cell::new(0));
    let element = layout! {
        Fragment {
            DefaultChildren($if = show_in_layout.get()) {
                Counter(.count = count.clone())
            }
        }
    };

    mount_root(&element);
    assert_eq!(count.get(), 0);

    set_show.set(true);
    assert_eq!(count.get(), 1);
}

#[test]
fn layout_if_directive_supports_components_without_props() {
    let element = layout! {
        Fragment {
            TransparentHost($if = true)
        }
    };

    mount_root(&element);
}

#[test]
fn layout_wrapper_directive_wraps_an_element() {
    let mounts = Rc::new(RefCell::new(Vec::new()));
    let count = Rc::new(Cell::new(0));
    let element = layout! {
        DefaultChildren($wrapper = RecordingProvider("theme", mounts.clone())) {
            Counter(.count = count.clone())
        }
    };

    mount_root(&element);

    assert_eq!(&*mounts.borrow(), &["theme"]);
    assert_eq!(count.get(), 1);
}

#[test]
fn layout_wrapper_directive_nests_multiple_wrappers_in_order() {
    let mounts = Rc::new(RefCell::new(Vec::new()));
    let count = Rc::new(Cell::new(0));
    let element = layout! {
        DefaultChildren(
            $wrapper = [
                RecordingProvider("theme", mounts.clone()),
                RecordingProvider("style", mounts.clone()),
            ],
        ) {
            Counter(.count = count.clone())
        }
    };

    mount_root(&element);

    assert_eq!(&*mounts.borrow(), &["theme", "style"]);
    assert_eq!(count.get(), 1);
}

#[test]
fn layout_macro_binds_host_handle_when_it_is_provided() {
    let stale_handle =
        nestix::Shared::from(Rc::new(String::from("stale")) as Rc<dyn std::any::Any>);
    let (host_handle, set_host_handle) = create_state(Some(stale_handle));
    let element = layout! {
        set_host_handle @ TransparentHost
    };

    assert!(host_handle.get().is_none());
    mount_root(&element);

    let handle = host_handle.get().expect("host handle should be bound");
    assert_eq!(handle.downcast::<String>().unwrap().as_str(), "host");
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
    let (value, set_value) = create_state(1);
    let observed = Rc::new(Cell::new(0));
    let element = nestix::create_element::<ScopedEffectComponent>(ScopedEffectComponentProps {
        value: value.clone(),
        observed: observed.clone(),
    });

    mount_root(&element);

    assert_eq!(observed.get(), 1);

    set_value.set(2);
    assert_eq!(observed.get(), 2);

    element.unmount();

    set_value.set(3);
    assert_eq!(observed.get(), 2);
}

#[test]
fn use_context_reads_context_from_the_current_element() {
    let observed = Rc::new(Cell::new(false));
    let element = layout! {
        ContextProvider::<
        String
        >(Rc::new("provided".to_string())) {
            ContextConsumer(.observed = observed.clone())
        }
    };

    mount_root(&element);

    assert!(observed.get());
}

#[test]
fn destructure_macro_derives_computed_signals_from_tuple_struct_and_named_struct_props() {
    let (data, set_data) = create_state(("key".to_string(), "value".to_string()));
    let (user, set_user) = create_state(DestructureUser {
        id: 7,
        name: "Ada".to_string(),
    });
    let (point, set_point) = create_state(DestructurePoint(3, 4));
    let (nested, set_nested) = create_state(((1, 2), DestructurePoint(3, 4)));
    let props = build_props!(DestructureProps(
        .data = data.clone(),
        .user = user.clone(),
        .point = point.clone(),
    ));

    destructure!((key, value) <- props.data);
    destructure!(DestructureUser { id, name: display_name } <- props.user);
    destructure!(DestructurePoint(x, y) <- props.point);
    destructure!(((nested_first, _), DestructurePoint(nested_x, nested_y)) <- nested);

    assert_eq!(key.get(), "key");
    assert_eq!(value.get(), "value");
    assert_eq!(id.get(), 7);
    assert_eq!(display_name.get(), "Ada");
    assert_eq!(x.get(), 3);
    assert_eq!(y.get(), 4);
    assert_eq!(nested_first.get(), 1);
    assert_eq!(nested_x.get(), 3);
    assert_eq!(nested_y.get(), 4);

    set_data.set(("next".to_string(), "item".to_string()));
    set_user.set(DestructureUser {
        id: 8,
        name: "Grace".to_string(),
    });
    set_point.set(DestructurePoint(5, 6));
    set_nested.set(((5, 6), DestructurePoint(7, 8)));

    assert_eq!(key.get(), "next");
    assert_eq!(value.get(), "item");
    assert_eq!(id.get(), 8);
    assert_eq!(display_name.get(), "Grace");
    assert_eq!(x.get(), 5);
    assert_eq!(y.get(), 6);
    assert_eq!(nested_first.get(), 5);
    assert_eq!(nested_x.get(), 7);
    assert_eq!(nested_y.get(), 8);
}

#[test]
fn destructure_macro_ignores_wildcards_and_struct_rest_patterns() {
    let (user, _set_user) = create_state(DestructureUser {
        id: 7,
        name: "Ada".to_string(),
    });
    let (pair, _set_pair) = create_state((1, 2));

    destructure!(DestructureUser { id, .. } <- user);
    destructure!((first, _) <- pair);

    assert_eq!(id.get(), 7);
    assert_eq!(first.get(), 1);
}
