# Fluent FSM

This crate was inspired by a C# library I use constantly called [Appccelerate State Machine](https://github.com/appccelerate/statemachine).

Fluent syntax is a natural way to describe state machines. Defacing code with a hundred macro decorations
and trait implementations is not.

These state machines are defined using a builder, built, then started
when ready to use.

## Quickstart

Create your states, events, and model. States and events are usually an enum,
and the model can be whatever you want. Then use the builder to describe your model.
Finally, call the appropriate `build_*()` method to create the machine, and `start()` the
machine when you're ready to start firing events. That's it!

```rust
use fluent_fsm::builder::*;
use fluent_fsm::passive::*;

fn main()
{
    let builder = 
        StateMachineBuilder::create(MyStates::Initial, MyModel::default())
            .on_enter_mut(|model| model.in_initial_state = true)
            .on(MyEvents::SomethingHappened, || { /* do stuff */ })
            .goto(MyStates::AnotherState)
            .in_state(MyStates::AnotherState)
            .on_enter_mut(|model| model.in_initial_state = false);

    let mut machine = builder.build_passive();

    assert!(machine.model().in_initial_state);
    machine.fire(MyEvents::SomethingHappened);
    assert!(!machine.model().in_initial_state);
}
```

See the tests in `lib.rs` for a simple example.

## Describing a machine

State machines have three generic type parameters that describe their functionality:

```rust
let game_engine: PassiveStateMachine<GameStates, GameEvents, GameModel>;
```

`TState` is the type which describes the finite states of the machine.

`TEvent` is the type which describes an external event that the machine may handle.

`TModel` is a representation of the internal state of the machine, and can be customized
to whatever you desire. You can access this state with the `model()` method.

## The builder

The `StateMachineBuilder` is a struct which uses fluent syntax to build up the
state machine. 

The machine starts in an initial state, with an initial internal model. You can
hook up closures or functions to run when a state is entered, exited, or an
event happens when in that state.

You can define transitions with the `goto` syntax. Whichever event was last mentioned in
an `on` function call is the event that is in scope for the transition. The `goto` function
can only be called once per event/state pair.

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

When the machine is defined, you can call `build_passive()` or `build_active()`
to select the machine operation.

```rust
let mut machine = builder.build_passive();
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

Contributions are welcome! Email me if you have any questions. Large features may
need some careful consideration, so ask me before you spend a lot of time on
them.

Desired features:

- Better documentation
- Better examples
- Static-only machines (no internal model)
- Looser type restrictions
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