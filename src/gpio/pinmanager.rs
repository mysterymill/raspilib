use std::{collections::{ HashMap }, rc::Rc, sync::{Arc, Mutex}};

use super::gpiopins::GpioPins;

type PortFrame = HashMap<GpioPins, gpio::GpioValue>;
type ChangeCallback = fn(before: PortFrame, now: PortFrame);

pub struct PinManager {
    output_ports: Vec<Arc<OutputPort>>,
    input_ports: Vec<Arc<InputPort>>,
}

lazy_static! {
    static ref PINMANAGER: Mutex<PinManager> = Mutex::new(PinManager::new()); 
}

impl <'l> PinManager {
    fn new() -> PinManager {
        PinManager {
            input_ports: vec![],
            output_ports: vec![],
        }
    }

    fn clear(&mut self) {
        self.output_ports.clear();
        self.input_ports.clear();
    }


    pub fn check_free_pins(&self, pins_to_check: &Vec<GpioPins>) -> Result<(), Vec<GpioPins>> {
        
        let input_pins = self.input_ports.iter().map(|port| port.get_PortFrame().keys()).flatten();
        let output_pins = self.output_ports.iter().map(|port| port.get_PortFrame().keys()).flatten();
        let taken_pins: Vec<&GpioPins> = input_pins.chain(output_pins).collect();


        let conflict_pins: Vec<GpioPins> = taken_pins.into_iter().filter(|taken_pin| pins_to_check.contains(taken_pin)).map(|pin| pin.clone()).collect();

        if conflict_pins.is_empty() { Ok(()) } else { Err(conflict_pins) }
    }

    pub fn register_OutputPort(&mut self, pins: Vec<GpioPins>) -> Result<Arc<OutputPort>, Vec<GpioPins>> {
        
        self.check_free_pins(&pins)?;
        

        let new_port = Arc::new(OutputPort::new(pins));
        {
            self.output_ports.push(new_port.clone());
        }

        Ok(new_port)
    }

    pub fn register_InputPort(&mut self, pins: Vec<GpioPins>) -> Result<InputPort, Vec<GpioPins>> {
        !unimplemented!()
    }
}

pub trait Port {
    fn get_PortFrame(&self) -> &PortFrame;
}

#[derive(Debug)]
pub struct OutputPort {
    state: PortFrame,
}

impl OutputPort {
    fn new(pins: Vec<GpioPins>) -> OutputPort {
        let mut state = PortFrame::new();
        pins.into_iter().for_each(|pin| {
            state.insert(pin, gpio::GpioValue::Low);
        });
        OutputPort { state }
    }
}

impl Port for OutputPort {
    fn get_PortFrame(&self) -> &PortFrame {
        &self.state
    }
}

#[derive(Debug)]
pub struct InputPort {
    state: PortFrame,
}

impl InputPort {
    fn new(pins: Vec<GpioPins>) -> InputPort {
        let mut state = PortFrame::new();
        pins.into_iter().for_each(|pin| {
            state.insert(pin, gpio::GpioValue::Low);
        });
        InputPort { state }
    }
}

impl Port for InputPort {
    fn get_PortFrame(&self) -> &PortFrame {
        &self.state
    }
}

#[cfg(test)]
mod test {
    use crate::gpio::{gpiopins::GpioPins::*, pinmanager::Port};

    #[test]
    fn check_free_pins_pins_free() {

        let check_result = super::PINMANAGER.lock().unwrap().check_free_pins(&vec![GPIO_01, GPIO_05, GPIO_11]);
        assert!(check_result.is_ok())
    }

    #[test]
    fn register_OutputPort_Ok() {
        let mut pinmanager = super::PINMANAGER.lock().unwrap();
        pinmanager.clear();
        let result = pinmanager.register_OutputPort(vec![GPIO_01, GPIO_05, GPIO_11]);
        assert!(result.is_ok());
        let new_port = result.unwrap();
        assert!(new_port.get_PortFrame().len() == 3);
    }

    #[test]
    fn register_OutputPort_fail() {
        let mut pinmanager = super::PINMANAGER.lock().unwrap();
        pinmanager.clear();
        let result_ok = pinmanager.register_OutputPort(vec![GPIO_01, GPIO_05, GPIO_11]);
        assert!(result_ok.is_ok());
        let result_err = pinmanager.register_OutputPort(vec![GPIO_01, GPIO_02, GPIO_06, GPIO_11, GPIO_13]);
        assert!(result_err.is_err());

        
        let error_pins = result_err.unwrap_err();
        assert!(error_pins.len() == 2);
        assert!(error_pins.contains(&GPIO_01));
        assert!(error_pins.contains(&GPIO_11));
    }
}