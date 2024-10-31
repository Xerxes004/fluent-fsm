# Fluent FSM

This crate was inspired by a C# library I use constantly called [Appccelerate State Machine](https://github.com/appccelerate/statemachine).

Fluent syntax is such a natural way to describe state machines. Defacing code with a hundred macro decorations
and trait implementations is not.

Here's how you create and use a state machine.

```Rust
use fluent_fsm::StateMachineBuilder;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

fn main()
    let state = Arc::new(Mutex::new(String::from("hello")));
    let on_event_state = Arc::clone(&state);
    let on_enter_state = Arc::clone(&state);
    let on_leave_state = Arc::clone(&state);

    lazy_static! {
        static ref static_state: Arc<Mutex<&'static str>> =
            Arc::new(Mutex::new("original static state"));
    }

    fn manipulate_static_state() -> anyhow::Result<()> {
        let mut s = static_state.lock().unwrap();
        *s = "manipulate static state!";
        Ok(())
    }

    let mut machine = StateMachineBuilder::create(0)
        .on_enter(move || {
            let mut g = on_enter_state.lock().unwrap();
            assert_eq!("hello", *g);
            g.push_str(" world");

            Ok(())
        })
        .on_leave(move || {
            let mut g = on_leave_state.lock().unwrap();
            assert_eq!("hello world from", *g);
            g.push_str(" state machine!");

            Ok(())
        })
        .on(1, move || {
            let mut g = on_event_state.lock().unwrap();
            assert_eq!("hello world", *g);
            g.push_str(" from");

            Ok(())
        })
        .on(1, manipulate_static_state)
        .goto(1)
        .unwrap()
        .build();

    machine.start().expect("start failed");

    assert_eq!(0, machine.current_state);

    machine.fire(1).expect("fire failed");

    assert_eq!(1, machine.current_state);

    assert_eq!("hello world from state machine!", *state.lock().unwrap());
    assert_eq!("manipulate static state!", *static_state.lock().unwrap());
}
```

