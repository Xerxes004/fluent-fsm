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

pub mod active;
pub mod builder;
pub mod passive;

#[cfg(test)]
mod tests {
    use super::builder::StateMachineBuilder;
    use Events::{AddEgg, CloseBasket, OpenBasket, TakeEgg};
    use States::{BasketClosed, BasketOpened};
    use std::sync::{Arc, Mutex};

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

        let mut machine = open_state_builder.build_passive();
        machine.start();

        // Initial state -- closed basket with a dozen eggs
        assert!(!machine.model().is_open);
        assert_eq!(machine.model().eggs, 12);

        // Try to add egg before it's open -- no change to egg count
        machine.fire(AddEgg);

        assert!(!machine.model().is_open);
        assert_eq!(machine.model().eggs, 12);

        // Open basket
        machine.fire(OpenBasket);

        assert!(machine.model().is_open);

        // Add egg to open basket
        machine.fire(AddEgg);

        assert_eq!(machine.model().eggs, 13);

        // Remove two eggs from open basket
        machine.fire(TakeEgg);
        machine.fire(TakeEgg);

        assert_eq!(machine.model().eggs, 11);

        // Close basket, restore egg count to one dozen upon exit
        machine.fire(CloseBasket);

        assert_eq!(machine.model().eggs, 12);
    }

    #[test]
    fn test_state_machine_static() {
        let builder = StateMachineBuilder::create(BasketClosed, ());

        let shared_model = Arc::new(Mutex::new(Basket {
            is_open: false,
            eggs: 12,
        }));

        let model1 = Arc::clone(&shared_model);
        let model2 = Arc::clone(&shared_model);
        let model3 = Arc::clone(&shared_model);
        let model4 = Arc::clone(&shared_model);
        let model5 = Arc::clone(&shared_model);

        let closed_state_builder = builder
            .on_enter(move || {
                let mut basket = model1.lock().unwrap();
                basket.is_open = false;
            })
            .on(OpenBasket, || {})
            .goto(BasketOpened);

        let open_state_builder = closed_state_builder
            .in_state(BasketOpened)
            .on_enter(move || {
                let mut basket = model2.lock().unwrap();
                basket.is_open = true;
            })
            .on(AddEgg, move || {
                let mut basket = model3.lock().unwrap();
                basket.eggs += 1;
            })
            .on(TakeEgg, move || {
                let mut basket = model4.lock().unwrap();
                basket.eggs -= 1;
            })
            .on_leave(move || {
                let mut basket = model5.lock().unwrap();
                basket.eggs = 12;
            })
            .on(CloseBasket, || {})
            .goto(BasketClosed);

        let mut machine = open_state_builder.build_passive();
        machine.start();

        {
            // Initial state -- closed basket with a dozen eggs
            let model = shared_model.lock().unwrap();
            assert!(!model.is_open);
            assert_eq!(model.eggs, 12);
        }

        // Try to add egg before it's open -- no change to egg count
        machine.fire(AddEgg);

        {
            let model = shared_model.lock().unwrap();
            assert!(!model.is_open);
            assert_eq!(model.eggs, 12);
        }

        // Open basket
        machine.fire(OpenBasket);

        {
            let model = shared_model.lock().unwrap();
            assert!(model.is_open);
        }

        // Add egg to open basket
        machine.fire(AddEgg);

        {
            let model = shared_model.lock().unwrap();
            assert_eq!(model.eggs, 13);
        }

        // Remove two eggs from open basket
        machine.fire(TakeEgg);
        machine.fire(TakeEgg);

        {
            let model = shared_model.lock().unwrap();
            assert_eq!(model.eggs, 11);
        }

        // Close basket, restore egg count to one dozen upon exit
        machine.fire(CloseBasket);

        {
            let model = shared_model.lock().unwrap();
            assert_eq!(model.eggs, 12);
        }
    }
}
