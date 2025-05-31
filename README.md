# fluent-fsm

Fluent syntax is a natural way to describe finite state machines. Defacing code with a hundred macro decorations
and trait implementations is not.

These state machines are defined using a builder, built, then started when ready to use. You can create a full state
machine in one expression!

```rust
fn create_turnstile_fsm() -> PassiveStateMachine<States, Turnstile, Events> {
    StateMachineBuilder::create(Locked, Turnstile::default())
        .on_enter(|| println!("turnstile is locked"))
        .on_mut(Coin, |model| {
            model.coins += 1;
            model.print_revenue()
        })
        .goto(Unlocked)
        .on(Push, || {
            println!("turnstile won't budge, maybe try inserting a coin")
        })
        .in_state(Unlocked)
        .on_enter(|| println!("turnstile clicks"))
        .on(Push, || println!("enjoy your ride!"))
        .on_mut(Push, |model| {
            model.riders += 1;
            model.print_ridership();
        })
        .goto(Locked)
        .on(Coin, || {
            println!("you already paid! Try pushing on the turnstile.")
        })
        .build_passive()
}
```

After the machine is built, start it, and then begin firing events to make it do stuff.

```rust
fn main() {
    let mut machine = create_turnstile_fsm();

    machine.start();

    loop {
        let action = input!("add a coin with (c) and push with (p): ");

        match action.trim().to_lowercase().parse() {
            Ok('c') => {
                machine.fire(Coin);
            }
            Ok('p') => {
                machine.fire(Push);
            }
            _ => continue,
        };
    }
}
```

## Features

* Fluent syntax for machine description
* No traits to implement -- machine can be defined and built in one line of code
* Define any number of actions for entry, exit, and event for every state. Actions are executed in order of definition.
* Built-in model manipulation
* Passive (blocking) or active (non-blocking) state machine
* No dependencies


## Quickstart

These are general guidelines, but feel free to mix and match how you see fit. 

### Passive machines

To see an example of a passive state machine, check out `examples/turnstile.rs`.

In general, passive machines rely on external events from `fire()` handled by `on()` and `on_mut()` followed by a `goto`
to define transitions.

The model is updated when events are fired, and transitions are based on fired events. This model is usually not shared
in other scopes; the user fires an event, then checks the model to see what changed.

### Active machines

To see an example of an active state machine, check out `examples/stoplight.rs`.

In general, active machines rely on the current state and the state of the model to define transitions, handled by the 
function passed to `create_active()`. This model is usually shared between scopes; the model is updated externally, then
the state machine checks the model for what state to transition to next.

## Contributions &amp; new features

Author: [Wes Kelly](https://github.com/Xerxes004)

If you want me to add a new feature, email me and we can work something out.

This crate was inspired by a C# library I use constantly called [Appccelerate State Machine](https://github.com/appccelerate/statemachine).

### Contributors

Contributions are welcome! Email me if you have any questions. Large features may
need some careful consideration, so ask me before you spend a lot of time on
them.

### Showcase

If you make something cool, please share it and I'll mention it here!

### Desired features

- Recursive events
- Error handling
- More state/event introspection to aid in logging and debugging
- Async interfaces
- FFI interface
- Better documentation
- Performance testing


## Versioning

Versions below 1.0.0 are considered to be fully experimental. Major things might change, including the license.

**Major releases** mean interface changes and functionality changes. Don't
switch between major versions unless you know why you want to.

**Minor releases** keep the existing interface the same, but may have additional
methods, traits, or features. The functionality may also be slightly different
behind the scenes, but should be non-intrusive.

**Patch releases** are for fixing bugs, and none of the interface will change.
