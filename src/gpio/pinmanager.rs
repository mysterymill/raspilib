use std::{sync::{Arc, Mutex}, collections::HashSet, iter::FromIterator, thread};

use gpio::GpioValue;

use super::gpiopins::GpioPins;

pub type PortDefinition<const I: usize> = [GpioPins; I]; // ToDo: Add where as soon as it is possible
pub type PortFrame<const I: usize> = [gpio::GpioValue; I];
pub type AllPinsOutputPort = OutputPort<26>;
pub type ChangeCallback<const I: usize> = fn(before: PortFrame<I>, now: PortFrame<I>);
pub type MismatchingPinsError = (String, Vec<GpioPins>);
pub type PinNotInPortError = String;

pub struct PinManager {
    pin_occupants: Vec<Arc<dyn PinOccupant + Send>>,
    active_ports: Arc<Mutex<Vec<Arc<dyn ActivePort>>>>,
}

lazy_static! {
    pub static ref PINMANAGER: Mutex<PinManager> = Mutex::new(PinManager::new()); 
}

impl <'l> PinManager {
    fn new() -> PinManager {
        let pin_manager = PinManager {
            pin_occupants: vec![],
            active_ports: Arc::new(Mutex::new(vec![])),
        };

        let active_ports = pin_manager.active_ports.clone();
        thread::spawn(|| {
            PinManager::activate_active_ports(active_ports);
        });

        pin_manager
    }

    pub fn clear(&mut self) {
        self.active_ports.lock().unwrap().clear();
        self.pin_occupants.clear();
    }

    pub fn activate_active_ports(ports_to_activate: Arc<Mutex<Vec<Arc<dyn ActivePort>>>>) {
        while true {
            let active_port_lock = ports_to_activate.lock().unwrap();

            active_port_lock.iter()
                .filter(|ap| !ap.is_paused())
                .for_each(|ap| ap.activate());

            std::mem::drop(active_port_lock);
            thread::yield_now();
        }
    }

    pub fn check_free_pins(&self, pins_to_check: &Vec<&GpioPins>) -> Result<(), MismatchingPinsError> {
        
        let taken_pins = self.pin_occupants.iter().map(|occupant| occupant.get_occupied_pins()).flatten();

        let conflict_pins: Vec<GpioPins> = taken_pins.into_iter().filter(|taken_pin| pins_to_check.contains(&taken_pin)).map(|pin| pin.clone()).collect();

        if conflict_pins.is_empty() { Ok(()) } else { Err(("Some pins have already been assigned".to_owned(), conflict_pins)) }
    }

    pub fn register_OutputPort<const I: usize>(&mut self, pins: &PortDefinition<I>) -> Result<Arc<OutputPort<I>>, MismatchingPinsError> {
        
        self.check_free_pins(&pins.into_iter().collect())?;
        

        let new_port = Arc::new(OutputPort::new(&pins)?);
        {
            self.pin_occupants.push(new_port.clone());
        }

        Ok(new_port)
    }

    pub fn register_InputPort<const I: usize>(&mut self, pins: &PortDefinition<I>) -> Result<Arc<InputPort<I>>, MismatchingPinsError> {
        
        self.check_free_pins(&pins.into_iter().collect())?;
        

        let new_port = Arc::new(InputPort::new(&pins)?);
        {
            self.pin_occupants.push(new_port.clone());
        }

        Ok(new_port)
    }

    pub fn add_active_port(&mut self, new_active_port: Arc<dyn ActivePort>) {
        self.active_ports.lock().unwrap().push(new_active_port);
    }
}


pub trait PinOccupant: Sync {
    fn get_occupied_pins(&self) -> HashSet<&GpioPins>;
}


pub trait Port<const I: usize>: Sized + PinOccupant {
    fn get_PortFrame(&self) -> &PortFrame<I>;
}

pub trait ActivePort: Sync + Send + 'static {
    fn start(self);
    fn stop(&self);
    fn pause(&mut self, paused: bool);
    fn is_paused(&self) -> bool;
    fn activate(&self);
}

pub trait WritablePort<const I: usize>: Port<I> {
    fn set_PortFrame(&mut self, new_state: PortFrame<I>);
    fn set_pin_state(&mut self, pin: usize, state: GpioValue) -> Result<GpioValue, PinNotInPortError>;
}

#[derive(Debug)]
pub struct OutputPort<const I: usize> {
    pins_of_port: PortDefinition<I>,
    state: PortFrame<I>,
}


impl <const I: usize> OutputPort<I> {
    fn new(pins: &PortDefinition<I>) -> Result<OutputPort<I>, MismatchingPinsError> {
        let mut known = HashSet::new();
        let mut duplicates = vec![];

        let mut pins_of_port = [GpioPins::GPIO_01; I];
        let state = [GpioValue::Low; I];

        for i in 0..I {
            let pin = pins[i];
            if !known.insert(pin.clone()) {
                duplicates.push(pin);
            } else {
                pins_of_port[i] = pin;
            }
        }

        if !duplicates.is_empty() {
            return Err(("Port definition has duplicate pins".to_owned(), duplicates));
        }


        Ok(OutputPort::<I> { pins_of_port, state })
    }
}

impl <const I: usize> PinOccupant for OutputPort<I> {
    fn get_occupied_pins(&self) -> HashSet<&GpioPins> {
        HashSet::from_iter(self.pins_of_port.iter())
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

    fn set_pin_state(&mut self, pin: usize, value: GpioValue) -> Result<GpioValue, PinNotInPortError> {
        if pin < 0 || pin >= I {
            Err("Pin is not part of port".to_owned())
        } else {
            let old_state = self.state[pin].clone();
            self.state[pin] = value;
            Ok(old_state)
        }
    }

}

#[derive(Debug)]
pub struct InputPort<const I: usize> {
    pins_of_port: PortDefinition<I>,
    state: PortFrame<I>,
}

impl <const I: usize> InputPort<I> {
    fn new(pins: &PortDefinition<I>) -> Result<InputPort<I>, MismatchingPinsError> {
        let mut known = HashSet::new();
        let mut duplicates = vec![];

        let mut pins_of_port = [GpioPins::GPIO_01; I];
        let state = [GpioValue::Low; I];

        for i in 0..I {
            let pin = pins[i];
            if !known.insert(pin.clone()) {
                duplicates.push(pin);
            } else {
                pins_of_port[i] = pin;
            }
        }

        if !duplicates.is_empty() {
            return Err(("Port definition has duplicate pins".to_owned(), duplicates));
        }

        Ok(InputPort::<I> { pins_of_port, state })
    }
}

impl <const I: usize> Port<I> for InputPort<I> {
    fn get_PortFrame(&self) -> &PortFrame<I> {
        &self.state
    }
}

impl <const I: usize>  PinOccupant for InputPort<I> {
    fn get_occupied_pins(&self) -> HashSet<&GpioPins> {
        HashSet::from_iter(self.pins_of_port.iter())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use crate::gpio::{gpiopins::GpioPins::*, pinmanager::Port};

    #[test]
    fn check_free_pins_pins_free() {

        let mut pinmanager = super::PINMANAGER.lock().unwrap();
        pinmanager.clear();

        let check_result = pinmanager.check_free_pins(&vec![&GPIO_01, &GPIO_05, &GPIO_11]);
        assert!(check_result.is_ok())
    }

    #[test]
    fn register_OutputPort_Ok() {
        let mut pinmanager = super::PINMANAGER.lock().unwrap();
        pinmanager.clear();
        let result = pinmanager.register_OutputPort(&[GPIO_01, GPIO_05, GPIO_11]);
        assert!(result.is_ok());
        let new_port = result.unwrap();
        assert!(new_port.get_PortFrame().len() == 3);
    }

    #[test]
    fn register_OutputPort_misdefined_fail() {
        let mut pinmanager = super::PINMANAGER.lock().unwrap();
        pinmanager.clear();
        let result_err = pinmanager.register_OutputPort(&[GPIO_01, GPIO_02, GPIO_06, GPIO_01, GPIO_13]);
        assert!(result_err.is_err());

        let error_pins = result_err.unwrap_err();
        assert!(error_pins.1.len() == 1);
        assert!(error_pins.1.contains(&GPIO_01));
    }

    #[test]
    fn register_OutputPort_conflict_fail() {
        let mut pinmanager = super::PINMANAGER.lock().unwrap();
        pinmanager.clear();
        let result_ok = pinmanager.register_OutputPort(&[GPIO_01, GPIO_05, GPIO_11]);
        assert!(result_ok.is_ok());
        let result_err = pinmanager.register_OutputPort(&[GPIO_01, GPIO_02, GPIO_06, GPIO_11, GPIO_13]);
        assert!(result_err.is_err());

        
        let error_pins = result_err.unwrap_err();
        assert!(error_pins.1.len() == 2);
        assert!(error_pins.1.contains(&GPIO_01));
        assert!(error_pins.1.contains(&GPIO_11));
    }


    #[test]
    fn register_InputPort_Ok() {
        let mut pinmanager = super::PINMANAGER.lock().unwrap();
        pinmanager.clear();
        let result = pinmanager.register_InputPort(&[GPIO_12, GPIO_10, GPIO_08, GPIO_06]);
        assert!(result.is_ok());
        let new_port = result.unwrap();
        assert!(new_port.get_PortFrame().len() == 4);
    }

    #[test]
    fn register_InputPort_misdefined_fail() {
        let mut pinmanager = super::PINMANAGER.lock().unwrap();
        pinmanager.clear();
        let result_err = pinmanager.register_InputPort(&[GPIO_04, GPIO_04, GPIO_04, GPIO_04]);
        assert!(result_err.is_err());

        let error_pins = result_err.unwrap_err();
        assert!(error_pins.1.len() == 3);
        assert!(error_pins.1.contains(&GPIO_04));
    }

    #[test]
    fn register_InputPort_conflict_fail() {
        let mut pinmanager = super::PINMANAGER.lock().unwrap();
        pinmanager.clear();
        let result_ok = pinmanager.register_OutputPort(&[GPIO_12, GPIO_10, GPIO_08, GPIO_06]);
        assert!(result_ok.is_ok());
        let result_err = pinmanager.register_InputPort(&[GPIO_01, GPIO_02, GPIO_06, GPIO_11, GPIO_13]);
        assert!(result_err.is_err());

        
        let error_pins = result_err.unwrap_err();
        assert!(error_pins.1.len() == 1);
        assert!(error_pins.1.contains(&GPIO_06));
    }
}