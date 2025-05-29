# Fluent FSM

This crate was inspired by a C# library I use constantly called [Appccelerate State Machine](https://github.com/appccelerate/statemachine).

Fluent syntax is such a natural way to describe state machines. Defacing code with a hundred macro decorations
and trait implementations is not.

Here's how you create and use a passive state machine.

```Rust
use fluent_fsm::builder::*;
use DoorEvents::*;
use DoorStates::*;

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
enum DoorStates {
    Closed,
    Opened,
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
enum DoorEvents {
    OpenDoor,
    CloseDoor,
}

struct DoorModel {
    door_open: bool,
}

fn door_simulation() {
    // Initial state: closed
    let builder = StateMachineBuilder::create(Closed, DoorModel { door_open: false })
        .on_enter_mut(|model: &mut DoorModel| {
            model.door_open = false;
        })
        .on(OpenDoor, || {
            println!("opening door");
        })
        .goto(Opened)
        .in_state(Opened)
        .on_enter_mut(|model: &mut DoorModel| {
            model.door_open = true;
        })
        .on(CloseDoor, || {
            println!("closing door");
        })
        .goto(Closed);

    let mut machine = builder.build();
    machine.start();

    assert_eq!(machine.model().door_open, false);

    machine.fire(OpenDoor);

    assert_eq!(machine.model().door_open, true);
}
```

