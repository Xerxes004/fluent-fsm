use crate::state_machine::PassiveStateMachine;
use std::fmt::Display;
use std::hash::Hash;

type ClosureBox<T> = Box<dyn Fn(&mut T)>;
type ClosureVec<T> = Vec<ClosureBox<T>>;

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

#[cfg(test)]
mod tests {
    use super::StateMachineBuilder;
    use crate::synchronous::tests::Events::{AddEgg, CloseBasket, OpenBasket, TakeEgg};
    use crate::synchronous::tests::States::{BasketClosed, BasketOpened};

    #[derive(Eq, PartialEq, Copy, Clone, Hash)]
    enum States {
        BasketClosed,
        BasketOpened,
    }

    #[derive(Eq, PartialEq, Copy, Clone, Hash)]
    enum Events {
        OpenBasket,
        AddEgg,
        TakeEgg,
        CloseBasket,
    }

    struct Basket {
        is_open: bool,
        eggs: u32,
    }

    #[test]
    fn test_state_machine_mut() {
        let builder = StateMachineBuilder::create(
            BasketClosed,
            Basket {
                is_open: false,
                eggs: 12,
            },
        );

        let closed_state_builder = builder
            .on_enter_mut(|basket: &mut Basket| {
                basket.is_open = false;
            })
            .on(OpenBasket, || {})
            .goto(BasketOpened);

        let open_state_builder = closed_state_builder
            .in_state(BasketOpened)
            .on_enter_mut(|basket: &mut Basket| {
                basket.is_open = true;
            })
            .on_mut(AddEgg, |basket: &mut Basket| {
                basket.eggs += 1;
            })
            .on_mut(TakeEgg, |basket: &mut Basket| {
                basket.eggs -= 1;
            })
            .on_leave_mut(|basket: &mut Basket| {
                basket.eggs = 12;
            })
            .on(CloseBasket, || {})
            .goto(BasketClosed);

        let mut machine = open_state_builder.build();

        assert!(!machine.model().is_open);
        assert_eq!(machine.model().eggs, 12);

        machine.start();

        assert!(!machine.model().is_open);
        assert_eq!(machine.model().eggs, 12);

        machine.fire(AddEgg);

        assert!(!machine.model().is_open);
        assert_eq!(machine.model().eggs, 12);

        machine.fire(OpenBasket);

        assert!(machine.model().is_open);

        machine.fire(AddEgg);

        assert_eq!(machine.model().eggs, 13);

        machine.fire(TakeEgg);
        machine.fire(TakeEgg);

        assert_eq!(machine.model().eggs, 11);

        machine.fire(CloseBasket);

        assert_eq!(machine.model().eggs, 12);
    }

    #[test]
    fn test_state_machine_static() {
        // let (tx, rx) = mpsc::channel();
        //
        // let tx_on_enter = tx.clone();
        // let tx_on_leave = tx.clone();
        // let tx_on_event = tx.clone();
        //
        // let builder = StateMachineBuilder::create(0u32, 0u32);
        //
        // let builder = builder
        //     .on_enter(move || {
        //         tx_on_enter
        //             .send(1)
        //             .with_context(|| "entering")
        //     })
        //     .on_leave(move || {
        //         tx_on_leave
        //             .send(3)
        //             .with_context(|| "leaving")
        //     })
        //     .on(1, move || {
        //         tx_on_event
        //             .send(2)
        //             .with_context(|| "sending OpenBasket")
        //     })
        //     .goto(BasketOpened).expect("transition already existed");
        //
        // let mut machine = builder.build();
        //
        // assert_eq!(machine.model().eggs, 0);
        // assert_eq!(machine.model().candy, 0);
        //
        // machine.start().expect("failed to start");
        //
        // assert_eq!(machine.model().eggs, 12);
        // assert_eq!(machine.model().candy, 0);
        //
        // machine.fire(OpenBasket).expect("failed to handle event");
        //
        // assert_eq!(machine.model().eggs, 10);
        // assert_eq!(machine.model().candy, 10);

        // let tx_on_enter = tx.clone();
        // let tx_on_leave = tx.clone();
        // let tx_on_event = tx.clone();
        //
        // let builder = builder.in_state(BasketOpened)
        //     .on_enter(move || {
        //         tx_on_enter.send("enter BasketClosed").with_context(|| "entering")
        //     })
        //     .on_enter_mut(|basket: &mut Basket| {
        //         basket.eggs = 2;
        //         basket.candy = 3;
        //         Ok(())
        //     })
    }
}
