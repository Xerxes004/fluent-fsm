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

        let mut machine = builder.build();
        machine.start();

        assert_eq!(machine.model().door_open, false);

        machine.fire(OpenDoor);

        assert_eq!(machine.model().door_open, true);
    }
}
