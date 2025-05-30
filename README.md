# Fluent FSM

This crate was inspired by a C# library I use constantly called [Appccelerate State Machine](https://github.com/appccelerate/statemachine).

Fluent syntax is such a natural way to describe state machines. Defacing code with a hundred macro decorations
and trait implementations is not.

These state machines are defined using a builder, built, then started
when ready to use.

## The builder

The `StateMachineBuilder` is a struct which uses fluent syntax to build up the
state machine. 

The machine starts in an initial state, with an initial internal model. You can
hook up closures or functions to run when a state is entered, exited, or an
event happens when in that state.

You can define transitions with the `goto` syntax.

```rust
// Assume the state machine is for a garage door.
builder
    .in_state(DoorClosing)
    .on_enter(|| {
        turn_on_light();
        start_motors();
    })
    .on_enter_mut(|model: &mut GarageDoor| { 
        model.closing = true;
    })
    .on_leave_mut(|model: &mut MyModel| {
        model.closing = false;
    })
    .on(BeamTripped, || { 
        emergency_open_door();
    })
    .goto(DoorOpening)
    .on_mut(DoorAtBottom, |model: &mut MyModel| { 
        model.closing = false;
    })
    .goto(DoorClosed);
```

When the machine is defined, you can call `build()` or `build_active()`
to select the machine operation.

```rust
let mut machine = builder.build();
// or
let mut machine = builder.build_active();

machine.start();
```

If you don't know which one to pick, start with passive.

## The machine

There are two ways to use the state machine: passive and active.

The passive state machine handles events atomically. That is, when you fire an event
with a passive state machine, it executes all events and transitions before
`fire()` returns.

The active state machine has a background thread which listens for new events using
an `mpsc::channel()`. Fire returns immediately, and the events and transitions are handled
whenever that thread gets around to it.

## Example

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


## Contributions &amp; new features

Contributions are welcome!

Desired features:

- Recursive events
- Async interface
- Performance testing


## Versioning

**Major releases** mean interface changes and functionality changes. Don't
switch between major versions unless you know why you want to.

**Minor releases** keep the existing interface the same, but may have new
methods, traits, or features. The functionality may also be slightly different
behind the scenes, but should be non-intrusive.

**Patch releases** are for fixing bugs, and none of the interface will change.

Release candidates, alphas, and betas will follow the same convention, but may
not be ready for production use.