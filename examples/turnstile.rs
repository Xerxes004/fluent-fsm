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
    let locked_state_builder = StateMachineBuilder::create(Locked, Turnstile::default())
        .on_enter(|| println!("turnstile is locked"))
        .on_mut(Coin, |model| {
            model.coins += 1;
            model.print_revenue()
        })
        .goto(Unlocked)
        .on(Push, || {
            println!("turnstile won't budge, maybe try inserting a coin")
        });

    let unlocked_state_builder = locked_state_builder
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
        });

    unlocked_state_builder.build_passive()
}
