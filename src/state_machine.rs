use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

type ClosureBox<T> = Box<dyn Fn(&mut T)>;
type ClosureVec<T> = Vec<ClosureBox<T>>;

#[derive(Debug, Copy, Clone)]
pub enum MachineAction<TState: Copy + Clone, TEvent: Copy + Clone> {
    Started {
        initial_state: TState,
    },
    Entered {
        from: TState,
        to: TState,
        because: TEvent,
    },
    Exited {
        from: TState,
        to: TState,
        because: TEvent,
    },
    HandledEvent {
        state: TState,
        event: TEvent,
    },
    Errored {
        state: TState,
    },
}

pub struct PassiveStateMachine<
    TEvent: Eq + Hash + Copy + Clone,
    TState: Eq + Hash + Copy + Clone,
    TModel,
> {
    running: bool,
    current_state: TState,
    model: TModel,

    on_event: HashMap<(TState, TEvent), ClosureVec<TModel>>,
    on_enter: HashMap<TState, ClosureVec<TModel>>,
    on_leave: HashMap<TState, ClosureVec<TModel>>,

    transitions: HashMap<(TState, TEvent), TState>,
}

impl<TEvent: Eq + Hash + Copy, TState: Eq + Hash + Copy, TModel>
    PassiveStateMachine<TEvent, TState, TModel>
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

    pub(crate) fn add_event_handler<F: 'static>(&mut self, state: TState, event: TEvent, func: F)
    where
        F: Fn(&mut TModel),
    {
        let key = (state, event);
        let handler: ClosureBox<TModel> = Box::new(func);
        match self.on_event.get_mut(&key) {
            Some(vec) => {
                vec.push(handler);
            }
            None => {
                self.on_event.insert(key, vec![handler]);
            }
        }
    }

    pub(crate) fn add_enter_handler<F: 'static>(&mut self, state: TState, func: F)
    where
        F: Fn(&mut TModel),
    {
        let handler: ClosureBox<TModel> = Box::new(func);
        match self.on_enter.get_mut(&state) {
            Some(vec) => {
                vec.push(handler);
            }
            None => {
                self.on_enter.insert(state, vec![handler]);
            }
        }
    }

    pub(crate) fn add_leave_handler<F: 'static>(&mut self, state: TState, func: F)
    where
        F: Fn(&mut TModel),
    {
        let handler: ClosureBox<TModel> = Box::new(func);
        match self.on_leave.get_mut(&state) {
            Some(vec) => {
                vec.push(handler);
            }
            None => {
                self.on_leave.insert(state, vec![handler]);
            }
        }
    }

    pub(crate) fn add_transition(&mut self, on: TEvent, from: TState, to: TState) {
        self.transitions.insert((from, on), to);
    }

    pub fn model(&self) -> &TModel {
        &self.model
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
            if let Some(actions) = self.on_leave.get(&(self.current_state)) {
                for action in actions.iter() {
                    action(&mut self.model);
                }
            }

            self.current_state = *state;

            if let Some(actions) = self.on_enter.get(&(self.current_state)) {
                for action in actions.iter() {
                    action(&mut self.model);
                }
            }
        }
    }
}
