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

use crate::Events::{Coin, Push};
use crate::States::{Locked, Unlocked};
use fluent_fsm::builder::StateMachineBuilder;
use fluent_fsm::passive::PassiveStateMachine;
use prompted::input;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum States {
    Locked,
    Unlocked,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum Events {
    Push,
    Coin,
}

struct Turnstile {
    coins: u32,
    riders: u32,
}

impl Default for Turnstile {
    fn default() -> Self {
        Self {
            coins: 0,
            riders: 0,
        }
    }
}

impl Turnstile {
    fn print_revenue(&self) {
        println!("revenue: ${:.2}", self.coins as f32 * 0.25);
    }
    fn print_ridership(&self) {
        println!("ridership: {}", self.riders);
    }
}

fn main() {
    let mut machine = create_turnstile_fsm();

    machine.start();

    loop {
        let action = input!("add a coin with (c) and push with (p): ");

        match action.trim().to_lowercase().parse() {
            Ok('c') => {
                machine.fire(Coin);
            }
            Ok('p') => {
                machine.fire(Push);
            }
            _ => continue,
        };
    }
}

fn create_turnstile_fsm() -> PassiveStateMachine<States, Turnstile, Events> {
    StateMachineBuilder::create(Locked, Turnstile::default())
        .on_enter(|| println!("turnstile is locked"))
        .on_mut(Coin, |model| {
            model.coins += 1;
            model.print_revenue()
        })
        .goto(Unlocked)
        .on(Push, || {
            println!("turnstile won't budge, maybe try inserting a coin")
        })
        .in_state(Unlocked)
        .on_enter(|| println!("turnstile clicks"))
        .on(Push, || println!("enjoy your ride!"))
        .on_mut(Push, |model| {
            model.riders += 1;
            model.print_ridership();
        })
        .goto(Locked)
        .on(Coin, || {
            println!("you already paid! Try pushing on the turnstile.")
        })
        .build_passive()
}
