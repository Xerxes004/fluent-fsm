use crate::active::ActiveStateMachine;
use crate::machine::passive::PassiveStateMachine;
use std::hash::Hash;

pub struct StateMachineBuilder<TState: Eq + Hash + Copy, TModel = (), TEvent: Eq + Hash + Copy = ()>
{
    working_on_state: TState,
    working_on_event: Option<TEvent>,
    current_state_machine: PassiveStateMachine<TState, TModel, TEvent>,
}

impl<TState, TModel, TEvent> StateMachineBuilder<TState, TModel, TEvent>
where
    TState: Eq + Hash + Copy + Sync + Send + 'static,
    TModel: Sync + Send + 'static,
    TEvent: Eq + Hash + Copy + Sync + Send + 'static,
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

    pub fn on_enter(self, func: impl Fn() + 'static + Sync + Send) -> Self {
        let mut builder = self;
        let machine = &mut builder.current_state_machine;
        machine.add_enter_handler(builder.working_on_state, move |_: &mut TModel| func());
        builder
    }

    /// Run the given function when the state specified by `in_state` is entered
    pub fn on_enter_mut(self, func: impl Fn(&mut TModel) + 'static + Sync + Send) -> Self {
        let mut builder = self;

        let machine = &mut builder.current_state_machine;

        machine.add_enter_handler(builder.working_on_state, func);

        builder
    }

    pub fn on_leave(self, func: impl Fn() + 'static + Sync + Send) -> Self {
        let wrapper = move |_: &mut TModel| func();
        self.on_leave_mut(wrapper)
    }

    /// Run the given function when the state specified by `in_state` is left
    pub fn on_leave_mut(self, func: impl Fn(&mut TModel) + 'static + Sync + Send) -> Self {
        let mut builder = self;

        let machine = &mut builder.current_state_machine;

        machine.add_leave_handler(builder.working_on_state, func);
        builder
    }

    pub fn on(self, event: TEvent, func: impl Fn() + 'static + Sync + Send) -> Self {
        let wrapper = move |_: &mut TModel| func();
        self.on_mut(event, wrapper)
    }

    /// Run the given function when the event is fired in the state specified by `in_state`
    pub fn on_mut(self, event: TEvent, func: impl Fn(&mut TModel) + 'static + Sync + Send) -> Self {
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

    /// Create a passive state machine, finalizing the builder
    pub fn build_passive(self) -> PassiveStateMachine<TState, TModel, TEvent> {
        self.current_state_machine
    }

    /// Create an active state machine, finalizing the builder
    pub fn build_active(
        self,
        tick: impl Fn(&TState, &TModel) -> Option<TState> + Send + Sync + 'static,
    ) -> ActiveStateMachine<TState, TModel, TEvent> {
        ActiveStateMachine::create(tick, self.current_state_machine)
    }
}
