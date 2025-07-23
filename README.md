# Nestix
A React-like declarative layout and state management library for Rust.

> [!WARNING]  
> This library is still in early development stages. Although all major features have been implemented, APIs can break at any point, 

> [!NOTE]  
> Just like you need `react-dom` or `react-native` to build an actual application, Nestix itself does not handle any platform-specific logics like rendering. If you want to know how to build an UI framework using Nestix, see [examples](#examples).

## Examples
For more examples, see [examples](./examples) folder.

### Counter
```rust
#[component]
fn Counter() -> Element {
    log::debug!("render Counter");

    let counter = state(|| 0);
    let increment = remember(|| {
        callback!(
            [counter] || {
                counter.update(|prev| *prev += 1);
            }
        )
    });

    layout! {
        FlexView(
            .direction = FlexDirection::Column,
            .width = 100.0
        ) {
            Text(counter.get().to_string()),
            Button(.on_click = increment.clone_shared()) {
                Text("Increment")
            },
        }
    }
}
```

## Documentation
WIP