use crate::machine::passive::PassiveStateMachine;
use std::hash::Hash;

pub struct StateMachineBuilder<TEvent: Eq + Hash + Copy, TState: Eq + Hash + Copy, TModel> {
    working_on_state: TState,
    working_on_event: Option<TEvent>,
    current_state_machine: PassiveStateMachine<TEvent, TState, TModel>,
}

impl<TEvent: Eq + Hash + Copy, TState: Eq + Hash + Copy, TModel>
    StateMachineBuilder<TEvent, TState, TModel>
{
    /// Create a state machine builder that starts in the given state
    pub fn create(initial_state: TState, initial_model: TModel) -> Self {
        Self {
            working_on_state: initial_state,
            working_on_event: None,
            current_state_machine: PassiveStateMachine::new(initial_state, initial_model),
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

    pub fn on_enter<F: 'static>(self, func: F) -> Self
    where
        F: Fn(),
    {
        let wrapper = move |_: &mut TModel| func();
        self.on_enter_mut(wrapper)
    }

    /// Run the given function when the state specified by `in_state` is entered
    pub fn on_enter_mut<F: 'static>(self, func: F) -> Self
    where
        F: Fn(&mut TModel),
    {
        let mut builder = self;

        let machine = &mut builder.current_state_machine;

        machine.add_enter_handler(builder.working_on_state, func);

        builder
    }

    pub fn on_leave<F: 'static>(self, func: F) -> Self
    where
        F: Fn(),
    {
        let wrapper = move |_: &mut TModel| func();
        self.on_leave_mut(wrapper)
    }

    /// Run the given function when the state specified by `in_state` is left
    pub fn on_leave_mut<F: 'static>(self, func: F) -> Self
    where
        F: Fn(&mut TModel),
    {
        let mut builder = self;

        let machine = &mut builder.current_state_machine;

        machine.add_leave_handler(builder.working_on_state, func);
        builder
    }

    pub fn on<F: 'static>(self, event: TEvent, func: F) -> Self
    where
        F: Fn(),
    {
        let wrapper = move |_: &mut TModel| func();
        self.on_mut(event, wrapper)
    }

    /// Run the given function when the event is fired in the state specified by `in_state`
    pub fn on_mut<F: 'static>(self, event: TEvent, func: F) -> Self
    where
        F: Fn(&mut TModel),
    {
        let mut builder = self;
        builder.working_on_event = Some(event);

        let machine = &mut builder.current_state_machine;

        machine.add_event_handler(builder.working_on_state, event, func);

        builder
    }

    /// Transition from the state specified by `in_state` to the given state when the event
    /// specified by `on` is fired.
    pub fn goto(self, state: TState) -> Self {
        let mut builder = self;

        match builder.working_on_event {
            Some(e) => {
                builder
                    .current_state_machine
                    .add_transition(e, builder.working_on_state, state);
                builder.working_on_event = None;
            }
            None => {
                panic!("Can't add a transition before an event is in scope with on()")
            }
        }

        builder
    }

    /// Finalize building of the state machine, and move the state machine out of the builder
    pub fn build(self) -> PassiveStateMachine<TEvent, TState, TModel> {
        self.current_state_machine
    }
}
