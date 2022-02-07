use std::{sync::{Arc, Mutex}};

use super::gpiopins::GpioPins;

type PortDefinition<const I: usize> = [GpioPins; I]; // ToDo: Add where as soon as it is possible
type PortFrame<const I: usize> = [gpio::GpioValue; I];
type ChangeCallback<const I: usize> = fn(before: PortFrame<I>, now: PortFrame<I>);
type MismatchingPinsError = Vec<GpioPins>;

pub struct PinManager {
    // ToDo: This is the Part I don't know how to solve yet
    output_ports: Vec<Arc<OutputPort<_>>>,
    input_ports: Vec<Arc<InputPort<_>>,
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


    pub fn check_free_pins(&self, pins_to_check: &Vec<GpioPins>) -> Result<(), MismatchingPinsError> {
        
        let input_pins = self.input_ports.iter().map(|port| port.get_PortFrame().keys()).flatten();
        let output_pins = self.output_ports.iter().map(|port| port.get_PortFrame().keys()).flatten();
        let taken_pins: Vec<&GpioPins> = input_pins.chain(output_pins).collect();


        let conflict_pins: Vec<GpioPins> = taken_pins.into_iter().filter(|taken_pin| pins_to_check.contains(taken_pin)).map(|pin| pin.clone()).collect();

        if conflict_pins.is_empty() { Ok(()) } else { Err(conflict_pins) }
    }

    pub fn register_OutputPort<const I: usize>(&mut self, pins: PortDefinition<I>) -> Result<Arc<OutputPort<I>>, Vec<GpioPins>> {
        
        self.check_free_pins(&pins)?;
        

        let new_port = Arc::new(OutputPort::new(pins));
        {
            self.output_ports.push(new_port.clone());
        }

        Ok(new_port)
    }

    pub fn register_InputPort<const I: usize>(&mut self, pins: PortDefinition<I>) -> Result<Arc<InputPort<I>>, MismatchingPinsError> {
        
        self.check_free_pins(&pins)?;
        

        let new_port = Arc::new(InputPort::new(pins));
        {
            self.input_ports.push(new_port.clone());
        }

        Ok(new_port)
    }
}

pub trait Port<const I: usize> {
    fn get_PortFrame(&self) -> &PortFrame<I>;
}

pub trait WritablePort<const I: usize>: Port<I> {
    fn set_PortFrame(&mut self, new_state: PortFrame<I>) -> Result<(), MismatchingPinsError>;
}

#[derive(Debug)]
pub struct OutputPort<const I: usize> {
    state: PortFrame<I>,
}

impl <const I: usize> OutputPort<I> {
    fn new(pins: Vec<GpioPins>) -> OutputPort<I> {
        let mut state = PortFrame::new();
        pins.into_iter().for_each(|pin| {
            state.insert(pin, gpio::GpioValue::Low);
        });
        OutputPort { state }
    }
}

impl <const I: usize> Port<I> for OutputPort<I> {
    fn get_PortFrame(&self) -> &PortFrame<I> {
        &self.state
    }
}

impl <const I: usize> WritablePort<I> for OutputPort<I> {
    fn set_PortFrame(&mut self, new_state: PortFrame<I>) {
        self.state = new_state;
    }
}

#[derive(Debug)]
pub struct InputPort<const I: usize> {
    state: PortFrame<I>,
}

impl <const I: usize> InputPort<I> {
    fn new(pins: Vec<GpioPins>) -> InputPort<I> {
        let mut state = PortFrame::new();
        pins.into_iter().for_each(|pin| {
            state.insert(pin, gpio::GpioValue::Low);
        });
        InputPort { state }
    }
}

impl <const I: usize> Port<I> for InputPort<I> {
    fn get_PortFrame(&self) -> &PortFrame<I> {
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


    #[test]
    fn register_InputPort_Ok() {
        let mut pinmanager = super::PINMANAGER.lock().unwrap();
        pinmanager.clear();
        let result = pinmanager.register_InputPort(vec![GPIO_12, GPIO_10, GPIO_08, GPIO_06]);
        assert!(result.is_ok());
        let new_port = result.unwrap();
        assert!(new_port.get_PortFrame().len() == 4);
    }

    #[test]
    fn register_InputPort_fail() {
        let mut pinmanager = super::PINMANAGER.lock().unwrap();
        pinmanager.clear();
        let result_ok = pinmanager.register_OutputPort(vec![GPIO_12, GPIO_10, GPIO_08, GPIO_06]);
        assert!(result_ok.is_ok());
        let result_err = pinmanager.register_InputPort(vec![GPIO_01, GPIO_02, GPIO_06, GPIO_11, GPIO_13]);
        assert!(result_err.is_err());

        
        let error_pins = result_err.unwrap_err();
        assert!(error_pins.len() == 1);
        assert!(error_pins.contains(&GPIO_06));
    }
}