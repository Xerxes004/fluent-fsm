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

use crate::States::*;
use fluent_fsm::active::ActiveStateMachine;
use fluent_fsm::builder::StateMachineBuilder;
use prompted::input;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
enum States {
    Red,
    Yellow,
    Green,
}

struct Model {
    car_detected: bool,
    light_change_time: SystemTime,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            car_detected: false,
            light_change_time: SystemTime::now(),
        }
    }
}

impl Model {
    pub fn time_elapsed(&self) -> Duration {
        self.light_change_time
            .elapsed()
            .expect("time went backwards, please inform the nearest physicist")
    }
}

fn main() {
    let shared_model = Arc::new(RwLock::new(Model::default()));

    let machine = create_stoplight_fsm(Arc::clone(&shared_model));

    input!("Press enter to start light cycle\n");

    machine.start();

    loop {
        {
            let mut model = shared_model.write().unwrap();
            model.car_detected = true;
        }

        input!();
    }
}

fn create_stoplight_fsm(
    model: Arc<RwLock<Model>>,
) -> ActiveStateMachine<States, Arc<RwLock<Model>>> {
    StateMachineBuilder::create(Red, model)
        .on_enter_mut(|model| {
            let mut model = model.write().unwrap();
            model.light_change_time = SystemTime::now();

            println!("Red light!");
        })
        .in_state(Green)
        .on_enter_mut(|model| {
            let mut model = model.write().unwrap();
            model.car_detected = false;
            model.light_change_time = SystemTime::now();

            println!("Green light!");
        })
        .in_state(Yellow)
        .on_enter_mut(|model| {
            let mut model = model.write().unwrap();
            model.light_change_time = SystemTime::now();

            println!("Yellow light!");
        })
        .build_active(|state, model| {
            let model = model.read().unwrap();

            match state {
                Red => {
                    if model.car_detected {
                        println!("Car detected, initiating green light...");
                        thread::sleep(Duration::from_secs(1));
                        Some(Green)
                    } else {
                        None
                    }
                }
                Yellow => {
                    if model.time_elapsed() > Duration::from_secs(3) {
                        Some(Red)
                    } else {
                        None
                    }
                }
                Green => {
                    if model.time_elapsed() > Duration::from_secs(4) {
                        Some(Yellow)
                    } else {
                        None
                    }
                }
            }
        })
}
