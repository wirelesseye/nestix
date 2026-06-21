# Nestix
A React-like declarative layout and state management library for Rust.

> [!WARNING]  
> This library is still in early stages of development. Although all major features have been implemented, APIs can break at any time.

> [!NOTE]  
> Just like you need `react-dom` or `react-native` to build an actual application, Nestix itself does not handle any platform-specific logics like rendering. If you want to know how to build an application or UI library using Nestix, see [examples](#examples).

## Examples
For more examples, see [examples](./examples) folder.

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

## Documentation
WIP