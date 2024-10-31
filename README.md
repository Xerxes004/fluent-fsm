# Fluent FSM

This crate was inspired by a C# library I use constantly called [Appccelerate State Machine](https://github.com/appccelerate/statemachine).

Fluent syntax is such a natural way to describe state machines. Defacing code with a hundred macro decorations
and trait implementations is not.

Here's how you create and use a state machine.

```Rust
use fluent_fsm::{StateMachine, StateMachineBuilder};
use MyStates::*;
use MyEvents::*;

#[derive(Copy, Clone)]
enum MyStates {
    Open,
    Closed
}

#[derive(Copy, Clone)]
enum MyEvents {
    LiftLid,
    CloseLid
}

fn main() {
    let mut machine =
        StateMachineBuilder::create(Closed)
            .on_enter(|_| println!("lid closed!")
                .on(LiftLid)
                .execute(|_| println!("lifting lid!"))
                .goto(Open)
            .in_state(Open)
                .on_enter(|_| println!("lid lifted!"))
                .on(CloseLid)
                .execute(|_| println!("closing lid!"))
                .goto(Closed)
            .build();

    // TODO: Add error handling
    machine.fire(LiftLid);
    machine.fire(CloseLid);
}
```

