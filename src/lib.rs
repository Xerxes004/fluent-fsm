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

pub(crate) mod machine;

pub use machine::active;
pub use machine::builder;
pub use machine::passive;

#[cfg(test)]
pub mod tests {
    use crate::builder::*;
    use DoorEvents::*;
    use DoorStates::*;

    #[derive(Eq, PartialEq, Copy, Clone, Hash)]
    enum DoorStates {
        Closed,
        Opened,
    }

    #[derive(Eq, PartialEq, Copy, Clone, Hash)]
    enum DoorEvents {
        OpenDoor,
        CloseDoor,
    }

    struct DoorModel {
        door_open: bool,
    }

    #[test]
    pub fn test_readme_example() {
        // Initial state: closed
        let builder = StateMachineBuilder::create(Closed, DoorModel { door_open: false })
            .on_enter_mut(|model: &mut DoorModel| {
                model.door_open = false;
            })
            .on(OpenDoor, || {
                println!("opening door");
            })
            .goto(Opened)
            .in_state(Opened)
            .on_enter_mut(|model: &mut DoorModel| {
                model.door_open = true;
            })
            .on(CloseDoor, || {
                println!("closing door");
            })
            .goto(Closed);

        let mut machine = builder.build_passive();
        machine.start();

        assert_eq!(machine.model().door_open, false);

        machine.fire(OpenDoor);

        assert_eq!(machine.model().door_open, true);
    }
}
