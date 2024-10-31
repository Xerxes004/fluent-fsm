use anyhow::{bail, Context};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

type ClosureBox = Box<dyn Fn() -> anyhow::Result<()>>;
type ClosureVec = Vec<ClosureBox>;

pub struct StateMachineBuilder<
    TEvent: Eq + Hash + Copy + Display,
    TState: Eq + Hash + Copy + Display,
> {
    working_on_state: TState,
    working_on_event: Option<TEvent>,

    current_state_machine: PassiveStateMachine<TEvent, TState>,
}

pub struct PassiveStateMachine<
    TEvent: Eq + Hash + Copy + Display,
    TState: Eq + Hash + Copy + Display,
> {
    running: bool,
    current_state: TState,

    on_event: HashMap<(TState, TEvent), Arc<Mutex<ClosureVec>>>,
    on_enter: HashMap<TState, Arc<Mutex<ClosureBox>>>,
    on_leave: HashMap<TState, Arc<Mutex<ClosureBox>>>,

    transitions: HashMap<(TState, TEvent), TState>,
}

impl<TEvent: Eq + Hash + Copy + Display, TState: Eq + Hash + Copy + Display>
    PassiveStateMachine<TEvent, TState>
{
    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.running {
            bail!("State machine already running");
        }

        self.running = true;

        if let Some(handler) = self.on_enter.get(&(self.current_state)) {
            handler.lock().unwrap()()
        } else {
            Ok(())
        }
    }

    pub fn fire(&mut self, event: TEvent) -> anyhow::Result<()> {
        if !self.running {
            bail!("State machine is not running");
        }

        let mut result = Ok(());

        if let Some(handlers) = self.on_event.get(&(self.current_state, event)) {
            for handler in handlers.lock().unwrap().iter() {
                if let Err(e) = handler() {
                    result = result.with_context(|| e)
                }
            }
        }

        if let Some(state) = self.transitions.get(&(self.current_state, event)) {
            if let Some(on_leave) = self.on_leave.get(&(self.current_state)) {
                if let Err(e) = on_leave.lock().unwrap()() {
                    result = result.with_context(|| e);
                }
            }

            self.current_state = *state;

            if let Some(on_enter) = self.on_enter.get(&(self.current_state)) {
                if let Err(e) = on_enter.lock().unwrap()() {
                    result = result.with_context(|| e);
                }
            }
        }

        result
    }
}

impl<TEvent: Eq + Hash + Copy + Display, TState: Eq + Hash + Copy + Display>
    StateMachineBuilder<TEvent, TState>
{
    /// Create a state machine builder that starts in the given state
    pub fn create(initial_state: TState) -> Self {
        Self {
            working_on_state: initial_state,
            working_on_event: None,
            current_state_machine: PassiveStateMachine {
                running: false,
                current_state: initial_state,
                on_event: HashMap::new(),
                on_enter: HashMap::new(),
                on_leave: HashMap::new(),
                transitions: HashMap::new(),
            },
        }
    }

    /// Change the builder context to operate on the given state
    pub fn in_state(self, state: TState) -> Self {
        Self {
            working_on_state: state,
            working_on_event: None,
            ..self
        }
    }

    /// Run the given function when the state specified by `in_state` is entered
    pub fn on_enter<F: 'static>(self, func: F) -> Self
    where
        F: Fn() -> anyhow::Result<()>,
    {
        let mut builder = self;

        let machine = &mut builder.current_state_machine;

        // Get the vector of handlers for this state & event
        match machine.on_enter.get(&(builder.working_on_state)) {
            // If handlers already exist, get a reference to it
            Some(vec) => {
                let mut handler = vec.lock().unwrap();
                *handler = Box::new(func);
            }
            // If handlers do not exist, create it and store a new reference to it
            None => {
                let cb: ClosureBox = Box::new(func);
                let handlers = Arc::new(Mutex::new(cb));
                let on_event = &mut machine.on_enter;
                on_event.insert(builder.working_on_state, handlers);
            }
        };

        builder
    }

    /// Run the given function when the state specified by `in_state` is left
    pub fn on_leave<F: 'static>(self, func: F) -> Self
    where
        F: Fn() -> anyhow::Result<()>,
    {
        let mut builder = self;

        let machine = &mut builder.current_state_machine;

        // Get the vector of handlers for this state & event
        match machine.on_leave.get(&(builder.working_on_state)) {
            // If handlers already exist, get a reference to it
            Some(vec) => {
                let mut handler = vec.lock().unwrap();
                *handler = Box::new(func);
            }
            // If handlers do not exist, create it and store a new reference to it
            None => {
                let cb: ClosureBox = Box::new(func);
                let handlers = Arc::new(Mutex::new(cb));
                let on_leave = &mut machine.on_leave;
                on_leave.insert(builder.working_on_state, handlers);
            }
        };

        builder
    }

    /// Run the given function when the event is fired in the state specified by `in_state`
    pub fn on<F: 'static>(self, event: TEvent, func: F) -> Self
    where
        F: Fn() -> anyhow::Result<()>,
    {
        let mut builder = self;
        builder.working_on_event = Some(event);

        let machine = &mut builder.current_state_machine;

        // Get the vector of handlers for this state & event
        let handlers = match machine.on_event.get(&(builder.working_on_state, event)) {
            // If handlers already exist, get a reference to it
            Some(vec) => Arc::clone(vec),
            // If handlers do not exist, create it and store a new reference to it
            None => {
                let handlers = Arc::new(Mutex::new(vec![]));
                let on_event = &mut machine.on_event;
                on_event.insert((builder.working_on_state, event), handlers.clone());
                handlers
            }
        };

        let mut handlers = handlers.lock().unwrap();
        handlers.push(Box::new(func));
        builder
    }

    /// Transition from the state specified by `in_state` to the given state when the event
    /// specified by `on` is fired.
    pub fn goto(self, state: TState) -> anyhow::Result<Self> {
        let mut builder = self;

        match builder.working_on_event {
            Some(e) => {
                builder.working_on_event = None;
                let machine = &mut builder.current_state_machine;
                if let Some(transition) = machine
                    .transitions
                    .insert((builder.working_on_state, e), state)
                {
                    bail!(
                        "goto was already defined for: in {} on {} goto {}",
                        builder.working_on_state,
                        e,
                        transition
                    )
                }
                Ok(builder)
            }
            None => bail!("cannot goto a state without an event: be sure to call on()"),
        }
    }

    /// Finalize building of the state machine, and move the state machine out of the builder
    pub fn build(self) -> PassiveStateMachine<TEvent, TState> {
        self.current_state_machine
    }
}

#[cfg(test)]
mod tests {
    use crate::data_structures::state_machine::StateMachineBuilder;
    use lazy_static::lazy_static;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_state_machine() {
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
}
