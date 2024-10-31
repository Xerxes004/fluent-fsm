use std::collections::HashMap;

enum TransitionType<TEvent: Copy + Clone + PartialEq> {
    OnEnter,
    OnEvent(TEvent),
    OnExit,
}

pub struct StateMachineBuilder<TState: Copy + Clone + PartialEq, TEvent: Copy + Clone + PartialEq> {
    working_with_state: TState,
    current_machine: StateMachine<TState, TEvent>,
}

pub struct StateMachine<TState: Copy + Clone + PartialEq, TEvent: Copy + Clone + PartialEq> {
    current_state: TState,

    handlers: HashMap<(TState, TEvent, TransitionType<TEvent>), Vec<fn() -> anyhow::Result<()>>>,
    transitions: HashMap<(TState, TEvent), TState>,
}

impl<TState: Copy + Clone + PartialEq, TEvent: Copy + Clone + PartialEq>
    StateMachineBuilder<TState, TEvent>
{
    pub fn create(initial_state: TState) -> Self {
        Self {
            working_with_state: initial_state,
            current_machine: StateMachine::create(initial_state),
        }
    }

    pub fn on_enter(self, action: fn() -> anyhow::Result<()>) {
        unimplemented!()
    }

    pub fn on(self, event: TEvent, action: fn() -> anyhow::Result<()>) {
        unimplemented!()
    }
}

impl<TState: Copy + Clone + PartialEq, TEvent: Copy + Clone + PartialEq>
    StateMachine<TState, TEvent>
{
    fn create(initial_state: TState) -> Self {
        Self {
            current_state: initial_state,
            handlers: HashMap::new(),
            transitions: HashMap::new(),
        }
    }

    fn in_state(self, state: TState) -> Self {
        todo!("move to builder")
    }

    fn on_enter(self, state: TState, action: fn() -> anyhow::Result<()>) -> Self {
        todo!("move to builder")
    }

    fn on_exit(self, state: TState, action: fn() -> anyhow::Result<()>) -> Self {
        todo!("move to builder")
    }

    fn on(self, event: TEvent, in_state: TState, goto: TState) -> Self {
        todo!("move to builder")
    }

    fn fire(&mut self, event: TEvent) -> anyhow::Result<()> {
        todo!("move to builder")
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::MyEvents::LiftLid;
    use crate::StateMachineBuilder;

    #[derive(Copy, Clone, PartialEq)]
    enum MyStates {
        Open,
        Closed,
    }

    #[derive(Copy, Clone, PartialEq)]
    enum MyEvents {
        LiftLid,
        ShutLid,
    }

    #[test]
    fn test_basic_state_machine() {
        let builder = StateMachineBuilder::create(MyStates::Closed).on(LiftLid, || {
            println!("lift lid");
            Ok(())
        });

        unimplemented!()
    }
}
