use std::collections::HashMap;

enum TransitionType<TEvent: Copy + Clone + Eq> {
    OnEnter,
    OnEvent(TEvent),
    OnExit,
}

pub struct StateMachineBuilder<
    TState: Copy + Clone + Eq,
    TEvent: Copy + Clone + Eq,
    TGlobalState: Send + Sync = (),
> {
    working_with_state: TState,
    current_machine: StateMachine<TState, TEvent, TGlobalState>,
}

pub struct StateMachine<
    TState: Copy + Clone + Eq,
    TEvent: Copy + Clone + Eq,
    TGlobalState: Send + Sync = (),
> {
    current_state: TState,
    global_state: Option<TGlobalState>,

    handlers: HashMap<
        (TState, TEvent, TransitionType<TEvent>),
        Vec<fn(TGlobalState) -> anyhow::Result<()>>,
    >,
    transitions: HashMap<(TState, TEvent), TState>,
}

impl<TState: Copy + Clone + Eq, TEvent: Copy + Clone + Eq, TGlobalState: Send + Sync>
    StateMachineBuilder<TState, TEvent, TGlobalState>
{
    pub fn create(initial_state: TState) -> Self {
        Self {
            working_with_state: initial_state,
            current_machine: StateMachine::create(initial_state),
        }
    }

    pub fn on_enter(self, action: fn(TGlobalState) -> anyhow::Result<()>) {
        unimplemented!()
    }
}

impl<TState: Copy + Clone + Eq, TEvent: Copy + Clone + Eq, TGlobalState: Send + Sync>
    StateMachine<TState, TEvent, TGlobalState>
{
    fn create(initial_state: TState) -> Self {
        Self {
            current_state: initial_state,
            global_state: None,
            handlers: HashMap::new(),
            transitions: HashMap::new(),
        }
    }

    fn with_global_state(self, global_state: TGlobalState) -> Self {
        Self {
            global_state: Some(global_state),
            ..self
        }
    }

    fn in_state(self, state: TState) -> Self {
        todo!("move to builder")
    }

    fn on_enter(self, state: TState, action: fn(TGlobalState) -> anyhow::Result<()>) -> Self {
        todo!("move to builder")
    }

    fn on_exit(self, state: TState, action: fn(TGlobalState) -> anyhow::Result<()>) -> Self {
        todo!("move to builder")
    }

    fn on(self, event: TEvent, in_state: TState, goto: TState) -> Self {
        todo!("move to builder")
    }

    fn fire(&mut self, event: TEvent) -> anyhow::Result<()> {
        todo!("move to builder")
    }
}
