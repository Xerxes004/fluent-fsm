// MIT License
//
// Copyright (c) 2024 Wes Kelly
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::active::ActiveMachineEvent::*;
use crate::passive::PassiveStateMachine;
use std::hash::Hash;
use std::sync::{Arc, RwLock, mpsc};
use std::thread;
use std::thread::JoinHandle;

enum ActiveMachineEvent<T: Eq + Hash + Copy> {
    Start,
    Stop,
    ExternalEvent(T),
}

pub struct ActiveStateMachine<TState, TModel = (), TEvent = ()>
where
    TState: Eq + Hash + Copy,
    TEvent: Eq + Hash + Copy,
{
    internal_state: Arc<RwLock<PassiveStateMachine<TState, TModel, TEvent>>>,
    machine_loop: JoinHandle<()>,
    tx: mpsc::Sender<ActiveMachineEvent<TEvent>>,
}

impl<TState, TModel, TEvent> ActiveStateMachine<TState, TModel, TEvent>
where
    TEvent: Eq + Hash + Copy + Sync + Send + 'static,
    TState: Eq + Hash + Copy + Sync + Send + 'static,
    TModel: Sync + Send + 'static,
{
    pub(crate) fn create(
        active_action: impl Fn(&TState, &TModel) -> Option<TState> + 'static + Send + Sync,
        machine: PassiveStateMachine<TState, TModel, TEvent>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let machine = Arc::new(RwLock::new(machine));
        let internal_state = Arc::clone(&machine);

        let machine_loop = thread::spawn(move || {
            loop {
                match rx.try_recv() {
                    Ok(Start) => {
                        let mut machine = machine.write().unwrap();
                        machine.start();
                    }
                    Ok(ExternalEvent(event)) => {
                        let mut machine = machine.write().unwrap();
                        machine.fire(event);
                    }
                    Ok(Stop) => {
                        return;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        let mut machine = machine.write().unwrap();
                        if let Some(state) = active_action(machine.current_state(), machine.model())
                        {
                            machine.goto(state);
                        }
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        return;
                    }
                }

                thread::yield_now();
            }
        });

        Self {
            internal_state,
            machine_loop,
            tx,
        }
    }

    pub fn fire(&self, event: TEvent) {
        self.tx.send(ExternalEvent(event)).unwrap();
    }

    pub fn start(&self) {
        self.tx.send(Start).unwrap();
    }

    pub fn stop(self) {
        self.tx.send(Stop).unwrap();
        self.machine_loop.join().unwrap();
    }

    pub fn write_model(&mut self, update: impl Fn(&mut TModel) + Send + Sync + 'static) {
        let mut model = self.internal_state.write().unwrap();
        update(model.model_mut())
    }

    pub fn read_state<R>(&self, read: impl Fn(&TModel) -> R) -> R {
        let state = self.internal_state.read().unwrap();
        read(state.model())
    }
}

#[cfg(test)]
mod tests {
    use super::super::builder::StateMachineBuilder;
    use super::*;
    use std::time::{Duration, SystemTime};

    struct Model<TState> {
        in_state: TState,
        num_transitions: u32,
        prev_state: Option<TState>,
        last_transition: SystemTime,
    }

    impl Model<u32> {
        pub fn new() -> Self {
            Self {
                in_state: 0,
                num_transitions: 0,
                prev_state: None,
                last_transition: SystemTime::now(),
            }
        }

        pub fn time_since_last_transition(&self) -> Duration {
            SystemTime::now()
                .duration_since(self.last_transition)
                .expect("time went backwards, please inform the nearest physicist")
        }
    }

    #[test]
    fn test_active_state_machine() {
        const STATE_1: u32 = 111;
        const STATE_2: u32 = 222;
        const MAX_TRANSITIONS: u32 = 5;

        let builder = StateMachineBuilder::<u32, Model<u32>>::create(STATE_1, Model::<u32>::new())
            .on_enter_mut(|model| {
                model.in_state = STATE_1;
                model.num_transitions += 1;
                model.last_transition = SystemTime::now();
            })
            .on_leave_mut(|model| {
                model.prev_state = Some(STATE_1);
            })
            .in_state(STATE_2)
            .on_enter_mut(|model| {
                model.in_state = STATE_2;
                model.num_transitions += 1;
                model.last_transition = SystemTime::now();
            })
            .on_leave_mut(|model| {
                model.prev_state = Some(STATE_2);
            });

        let machine = builder.build_active(tick);
        machine.start();

        thread::sleep(Duration::from_millis(50));

        assert_eq!(
            machine.read_state(|model| model.num_transitions),
            MAX_TRANSITIONS
        );

        machine.stop();

        fn tick(state: &u32, model: &Model<u32>) -> Option<u32> {
            if model.num_transitions >= MAX_TRANSITIONS {
                return None;
            }

            match state {
                &STATE_1 => {
                    if let Some(prev) = model.prev_state {
                        assert_eq!(prev, STATE_2)
                    }

                    if model.time_since_last_transition() > Duration::from_millis(5) {
                        Some(STATE_2)
                    } else {
                        None
                    }
                }
                &STATE_2 => {
                    assert_eq!(model.prev_state, Some(STATE_1));

                    if model.time_since_last_transition() > Duration::from_millis(5) {
                        Some(STATE_1)
                    } else {
                        None
                    }
                }
                v => panic!("unexpected state: {v}"),
            }
        }
    }
}
