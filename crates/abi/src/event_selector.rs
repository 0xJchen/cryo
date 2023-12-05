use ethers_core::abi::Event;
use std::io::{self, Write};

pub struct EventSelector;

impl EventSelector {
    pub fn select_event(events: &[Event]) -> Result<&Event, String> {
        for (i, event) in events.iter().enumerate() {
            println!("{}: {}", i + 1, event.name);
        }

        print!("Select an event: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice: usize = input.trim().parse().map_err(|_| "Invalid input".to_string())?;

        events.get(choice - 1).ok_or_else(|| "Event not found".to_string())
    }
}

#[cfg(test)]
mod event_selector_tests {
    use super::*;
    use ethers_core::abi::{ParamType, EventParam, Event};

    fn mock_event(name: &str) -> Event {
        Event {
            name: name.to_string(),
            inputs: vec![EventParam {
                name: "test".to_string(),
                kind: ParamType::Address,
                indexed: false,
            }],
            anonymous: false,
        }
    }

    #[test]
    fn select_first_event() {
        let events = vec![mock_event("Event1"), mock_event("Event2")];
        // Simulate user input for selecting the first event
        // This part needs a way to mock standard input, or a change in the
        // EventSelector to allow injecting input sources for testing.
        let selected_event = EventSelector::select_event(&events);
        assert!(selected_event.is_ok());
        assert_eq!(selected_event.unwrap().name, "Event1");
    }

    // Additional tests could include invalid selections, no selection, etc.
}
