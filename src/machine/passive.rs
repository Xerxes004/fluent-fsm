use std::collections::HashMap;
use std::hash::Hash;

pub struct PassiveStateMachine<TState, TModel = (), TEvent = ()>
where
    TState: Eq + Hash + Copy + Clone,
    TEvent: Eq + Hash + Copy + Clone,
{
    running: bool,
    current_state: TState,
    model: TModel,

    on_event: HashMap<(TState, TEvent), Vec<Box<dyn Fn(&mut TModel) + 'static + Sync + Send>>>,
    on_enter: HashMap<TState, Vec<Box<dyn Fn(&mut TModel) + 'static + Sync + Send>>>,
    on_leave: HashMap<TState, Vec<Box<dyn Fn(&mut TModel) + 'static + Sync + Send>>>,

    transitions: HashMap<(TState, TEvent), TState>,
}

impl<TState, TModel, TEvent> PassiveStateMachine<TState, TModel, TEvent>
where
    TState: Eq + Hash + Copy,
    TEvent: Eq + Hash + Copy,
{
    pub(crate) fn new(initial_state: TState, model: TModel) -> Self {
        Self {
            running: false,
            current_state: initial_state,
            model,
            on_event: HashMap::new(),
            on_enter: HashMap::new(),
            on_leave: HashMap::new(),
            transitions: HashMap::new(),
        }
    }

    pub(crate) fn add_event_handler(
        &mut self,
        state: TState,
        event: TEvent,
        func: impl Fn(&mut TModel) + 'static + Sync + Send,
    ) {
        let key = (state, event);
        match self.on_event.get_mut(&key) {
            Some(vec) => {
                vec.push(Box::new(func));
            }
            None => {
                self.on_event.insert(key, vec![Box::new(func)]);
            }
        }
    }

    pub(crate) fn add_enter_handler(
        &mut self,
        state: TState,
        func: impl Fn(&mut TModel) + 'static + Sync + Send,
    ) {
        match self.on_enter.get_mut(&state) {
            Some(vec) => {
                vec.push(Box::new(func));
            }
            None => {
                self.on_enter.insert(state, vec![Box::new(func)]);
            }
        }
    }

    pub(crate) fn add_leave_handler(
        &mut self,
        state: TState,
        func: impl Fn(&mut TModel) + 'static + Sync + Send,
    ) {
        match self.on_leave.get_mut(&state) {
            Some(vec) => {
                vec.push(Box::new(func));
            }
            None => {
                self.on_leave.insert(state, vec![Box::new(func)]);
            }
        }
    }

    pub(crate) fn add_transition(&mut self, on: TEvent, from: TState, to: TState) {
        self.transitions.insert((from, on), to);
    }

    pub fn current_state(&self) -> &TState {
        &self.current_state
    }

    pub fn model(&self) -> &TModel {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut TModel {
        &mut self.model
    }

    pub fn start(&mut self) {
        if self.running {
            return;
        }

        self.running = true;

        if let Some(actions) = self.on_enter.get(&(self.current_state)) {
            for action in actions.iter() {
                action(&mut self.model);
            }
        }
    }

    pub fn fire(&mut self, event: TEvent) {
        if !self.running {
            panic!("State machine is not running");
        }

        // Handle event and update state
        if let Some(handlers) = self.on_event.get(&(self.current_state, event)) {
            for handler in handlers.iter() {
                handler(&mut self.model);
            }
        }

        // If a transition happens, handle on-leave and on-enter
        if let Some(state) = self.transitions.get(&(self.current_state, event)) {
            self.goto(*state);
        }
    }

    pub(crate) fn goto(&mut self, state: TState) {
        if let Some(actions) = self.on_leave.get(&(self.current_state)) {
            for action in actions.iter() {
                action(&mut self.model);
            }
        }

        self.current_state = state;

        if let Some(actions) = self.on_enter.get(&(self.current_state)) {
            for action in actions.iter() {
                action(&mut self.model);
            }
        }
    }
}
