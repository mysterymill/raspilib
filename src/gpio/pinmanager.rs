use std::collections::{ HashMap };

use super::gpiopins::GpioPins;

type PortFrame = HashMap<GpioPins, gpio::GpioValue>;
type ChangeCallback = fn(before: PortFrame, now: PortFrame);

pub struct PinManager<'p> {
    // Not sure yet, but it might be correct to take ownership here
    output_ports: Vec<&'p OutputPort>,
    input_ports: Vec<&'p InputPort>,
}

lazy_static! {
    static ref PINMANAGER: PinManager<'static> = PinManager::new(); 
}

impl <'p> PinManager<'p> {
    fn new() -> PinManager<'p> {
        PinManager {
            input_ports: vec![],
            output_ports: vec![],
        }
    }

    pub fn check_free_pins(&self, pins_to_check: Vec<GpioPins>) -> Result<(), Vec<&GpioPins>> {
        
        let input_pins = self.input_ports.iter().map(|port| port.get_PortState().keys()).flatten();
        let output_pins = self.output_ports.iter().map(|port| port.get_PortState().keys()).flatten();
        let taken_pins: Vec<&GpioPins> = input_pins.chain(output_pins).collect();


        let conflict_pins: Vec<&GpioPins> = taken_pins.into_iter().filter(|taken_pin| pins_to_check.contains(taken_pin)).collect();

        if conflict_pins.is_empty() { Ok(()) } else { Err(conflict_pins) }
    }
}

pub trait Port {
    fn get_PortState(&self) -> &PortFrame;
}

pub struct OutputPort {
    state: PortFrame,
}

impl Port for OutputPort {
    fn get_PortState(&self) -> &PortFrame {
        &self.state
    }
}

pub struct InputPort {
    state: PortFrame,
}

impl Port for InputPort {
    fn get_PortState(&self) -> &PortFrame {
        &self.state
    }
}

#[cfg(test)]
mod test {
    use crate::gpio::gpiopins::GpioPins::*;

    #[test]
    fn check_free_pins_pins_free() {

        let check_result = super::PINMANAGER.check_free_pins(vec![GPIO_01, GPIO_05, GPIO_11]);
        assert!(check_result.is_ok())
    }
}