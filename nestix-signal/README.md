# nestix-signal

`nestix-signal` is the standalone reactive runtime used by Nestix. It provides
mutable state, lazily cached computed values, reactive effects, read-only signal
handles, and utilities for controlling dependency tracking.

The crate is intentionally decoupled from `nestix`. You can use it on its own
without pulling in the component, layout, or macro APIs from the main Nestix
crate.

> [!WARNING]
> This library is still in early stages of development. APIs can break at any
> time.

## Features

- `create_state` creates mutable reactive values.
- `computed` derives lazily evaluated values from other signals.
- `effect` runs side effects immediately and reruns them when tracked
  dependencies change.
- `Readonly` exposes any signal through a cloneable read-only handle.
- `Signal` supports generic and boxed readable signal values.
- `untrack` reads signals without subscribing the current effect or computed
  value to future updates.

## Installation

From this workspace, depend on the crate directly:

```toml
[dependencies]
nestix-signal = { path = "../nestix-signal" }
```

When used from another workspace, point the path at this package or depend on
the published crate once one is available.

## Basic Usage

```rust
use std::{cell::Cell, rc::Rc};

use nestix_signal::{computed, create_state, effect};

let count = create_state(1);
let doubled = computed({
    let count = count.clone();
    move || count.get() * 2
});

let observed = Rc::new(Cell::new(0));
let _handle = effect({
    let doubled = doubled.clone();
    let observed = observed.clone();
    move || observed.set(doubled.get())
});

assert_eq!(observed.get(), 2);

count.set(2);

assert_eq!(observed.get(), 4);
```

Reading a signal inside `effect` or `computed` records a dependency. Updating
that signal notifies the dependent computations. Computed values are lazy: they
are marked dirty when dependencies change, then reevaluated the next time they
are read.

## State Updates

```rust
use nestix_signal::create_state;

let items = create_state(vec![1, 2]);

items.update(|items| {
    let mut next = items.clone();
    next.push(3);
    next
});

items.mutate(|items| items.push(4));

assert_eq!(items.get(), vec![1, 2, 3, 4]);
```

`State::set` notifies dependents only when the new value is different.
`State::set_unchecked`, `State::update`, and `State::mutate` always notify after
storing the new value.

## Effects

```rust
use std::{cell::Cell, rc::Rc};

use nestix_signal::{create_state, effect};

let count = create_state(0);
let runs = Rc::new(Cell::new(0));

let handle = effect({
    let count = count.clone();
    let runs = runs.clone();
    move || {
        count.get();
        runs.set(runs.get() + 1);
    }
});

count.set(1);
assert_eq!(runs.get(), 2);

handle.cancel();
count.set(2);
assert_eq!(runs.get(), 2);
```

Dropping an `EffectHandle` does not cancel the effect. Call
`EffectHandle::cancel` when the effect should stop rerunning and unsubscribe
from its dependencies.

## Read-only Signals

```rust
use nestix_signal::{Readonly, create_state};

let state = create_state("ready".to_string());
let readonly: Readonly<String> = state.clone().into_readonly();

assert_eq!(readonly.get(), "ready");
```

Use `Readonly` when a caller should be able to observe a value without being
able to mutate the underlying state through that handle.

## Debug Configuration

`debug_signals` enables debug-only runtime checks. In release builds it has no
effect.

```rust
use nestix_signal::{DebugConfig, debug_signals};

debug_signals(DebugConfig {
    detect_cyclic: true,
});
```

## License

This crate is part of the Nestix workspace. See the workspace license for
details.
