# Nestix

Nestix is a React-like declarative layout and state management library for
Rust. This repository contains the core, renderer-agnostic Nestix workspace:

- `nestix`: the main public API for components, elements, props, layouts, and
  built-in structural components.
- `nestix-signal`: the reactive runtime for state, computed values, effects,
  readonly signals, and shared callback/handle pointers.
- `nestix-macros`: the procedural macros behind `#[component]`, `#[props]`,
  `layout!`, `callback!`, `computed!`, and related syntax.
- `examples`: small applications and renderer examples that show how Nestix can
  be used from real Rust code.

> [!WARNING]  
> This library is still in early stages of development. Although all major features have been implemented, APIs can break at any time.

> [!NOTE]  
> Just like you need `react-dom` or `react-native` to build an actual application, Nestix itself does not handle platform-specific rendering. It manages component trees, reactive state, props, layout, and lifecycle; a renderer or UI component library supplies host components for the DOM, native views, terminal widgets, or another target.
>
> For building actual applications, see [`nestix-native`](https://github.com/wirelesseye/nestix-native).

## Key Features

- React-like component model for building declarative Rust UI trees.
- Fine-grained reactive state with signals, computed values, effects, and
  shared callbacks.
- Renderer-agnostic core that can target native views, DOM-like renderers,
  terminal UIs, or custom hosts.
- Procedural macros for ergonomic components, props, layouts, callbacks, and
  derived state.
- Built-in structural components and layout primitives for composing reusable
  interfaces.

## Documentation

See the [Nestix wiki](https://github.com/wirelesseye/nestix/wiki) for
comprehensive documentation, including getting started guides, core concepts,
signals, components, props, layout syntax, renderer integration, examples, and
API references.

## Examples

For more examples, see the [examples](./examples) folder.

### Counter

```rust
#[component]
fn Counter() -> Element {
    let count = create_state(0);

    layout! {
        FlexView {
            Label(.text = computed!([count] || format!("Count: {}", count.get())))
            Button(
                .title = "Click",
                .on_click = callback!([count] || {
                    count.mutate(|count| *count += 1);
                })
            )
            if count.get() % 2 == 0 {
                Label(.text = "Is Even!")
            }
        }
    }
}
```
