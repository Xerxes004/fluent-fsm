use crate::active::MachineCommand::*;
use crate::passive::PassiveStateMachine;
use std::hash::Hash;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;

enum MachineCommand<T: Eq + Hash + Copy> {
    Start,
    Stop,
    NewEvent(T),
}

pub struct ActiveStateMachine<TEvent, TState, TModel>
where
    TEvent: Eq + Hash + Copy,
    TState: Eq + Hash + Copy,
{
    internal_state: Arc<RwLock<PassiveStateMachine<TEvent, TState, TModel>>>,
    machine_loop: JoinHandle<()>,
    tx: mpsc::Sender<MachineCommand<TEvent>>,
}

impl<TEvent, TState, TModel> ActiveStateMachine<TEvent, TState, TModel>
where
    TEvent: Eq + Hash + Copy + Sync + Send + 'static,
    TState: Eq + Hash + Copy + Sync + Send + 'static,
    TModel: Sync + Send + 'static,
{
    pub(crate) fn create(machine: PassiveStateMachine<TEvent, TState, TModel>) -> Self {
        let (tx, rx) = mpsc::channel();
        let machine = Arc::new(RwLock::new(machine));
        let internal_state = Arc::clone(&machine);

        let machine_loop = thread::spawn(move || loop {
            match rx.try_recv() {
                Ok(Start) => {
                    let mut machine = machine.write().unwrap();
                    machine.start();
                }
                Ok(NewEvent(event)) => {
                    let mut machine = machine.write().unwrap();
                    machine.fire(event);
                }
                Ok(Stop) => {
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // Do nothing
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    return;
                }
            }

            thread::yield_now();
        });

        Self {
            internal_state,
            machine_loop,
            tx,
        }
    }

    pub fn fire(&self, event: TEvent) {
        self.tx.send(NewEvent(event)).unwrap();
    }

    pub fn start(&self) {
        self.tx.send(Start).unwrap();
    }

    pub fn stop(self) {
        self.tx.send(Stop).unwrap();
        self.machine_loop.join().unwrap();
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
    use std::time::Duration;

    #[test]
    fn test_active_state_machine() {
        let (tx, rx) = mpsc::channel();
        let tx2 = tx.clone();
        let add_and_send = move |num: &mut u32| {
            *num += 1;
            tx.send(*num).unwrap();
        };

        let builder = StateMachineBuilder::create(0, 0)
            .on_enter_mut(add_and_send.clone())
            .on_mut(123, add_and_send.clone())
            .on_leave_mut(add_and_send.clone())
            .goto(321)
            .in_state(321)
            .on_enter(move || tx2.send(0).unwrap());

        let machine = builder.build_active();
        machine.start();

        assert_eq!(rx.recv_timeout(Duration::from_millis(1000)).unwrap(), 1);
        assert_eq!(machine.read_state(|s| *s), 1);

        machine.fire(123);

        assert_eq!(rx.recv_timeout(Duration::from_millis(1000)).unwrap(), 2);
        assert_eq!(rx.recv_timeout(Duration::from_millis(1000)).unwrap(), 3);
        assert_eq!(rx.recv_timeout(Duration::from_millis(1000)).unwrap(), 0);

        assert_eq!(machine.read_state(|s| *s), 3);

        machine.stop()
    }
}
