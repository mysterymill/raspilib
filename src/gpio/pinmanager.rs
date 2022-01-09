use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use std::{convert::{TryFrom}};
#[allow(non_camel_case_types)]
#[derive(PartialEq, Debug, EnumIter)]
pub enum GpioPins {
    GPIO_01, GPIO_02, GPIO_03, GPIO_04, GPIO_05, GPIO_06, GPIO_07, GPIO_08, GPIO_09, GPIO_10,
    GPIO_11, GPIO_12, GPIO_13, GPIO_14, GPIO_15, GPIO_16, GPIO_17, GPIO_18, GPIO_19, GPIO_20,
    GPIO_21, GPIO_22, GPIO_23, GPIO_24, GPIO_25, GPIO_26
}

impl TryFrom<u8> for GpioPins {
    type Error = &'static str;

    fn try_from(pin_value: u8) -> Result<Self, Self::Error> {

        for pin in GpioPins::iter() {
            if pin_value == (&pin).into() {
                return Ok(pin);
            }
        }

        Err("Pin number is not a valid GPIO pin number (1-26)")
        
    }
}

impl From<&GpioPins> for u8 {
    fn from(pin: &GpioPins) -> u8 {
        match pin {
            GpioPins::GPIO_01 => 1,
            GpioPins::GPIO_02 => 2,
            GpioPins::GPIO_03 => 3,
            GpioPins::GPIO_04 => 4,
            GpioPins::GPIO_05 => 5,
            GpioPins::GPIO_06 => 6,
            GpioPins::GPIO_07 => 7,
            GpioPins::GPIO_08 => 8,
            GpioPins::GPIO_09 => 9,
            GpioPins::GPIO_10 => 10,
            GpioPins::GPIO_11 => 11,
            GpioPins::GPIO_12 => 12,
            GpioPins::GPIO_13 => 13,
            GpioPins::GPIO_14 => 14,
            GpioPins::GPIO_15 => 15,
            GpioPins::GPIO_16 => 16,
            GpioPins::GPIO_17 => 17,
            GpioPins::GPIO_18 => 18,
            GpioPins::GPIO_19 => 19,
            GpioPins::GPIO_20 => 20,
            GpioPins::GPIO_21 => 21,
            GpioPins::GPIO_22 => 22,
            GpioPins::GPIO_23 => 23,
            GpioPins::GPIO_24 => 24,
            GpioPins::GPIO_25 => 25,
            GpioPins::GPIO_26 => 26,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::{TryInto, TryFrom};

    use crate::gpio::pinmanager::GpioPins;

    #[test]
    fn gpio_pins_from_u8_ok() {
        let gpio_result: Result<GpioPins, &'static str> = (5 as u8).try_into();
        assert!(gpio_result.is_ok());
        assert_eq!(gpio_result.unwrap(), GpioPins::GPIO_05);

        let gpio_result: Result<GpioPins, &'static str> = GpioPins::try_from(8 as u8);
        assert!(gpio_result.is_ok());
        assert_eq!(gpio_result.unwrap(), GpioPins::GPIO_08);
    }

    #[test]
    fn gpio_pins_from_u8_err() {
        let gpio_result: Result<GpioPins, &'static str> = (35 as u8).try_into();
        assert!(gpio_result.is_err());
    }

    #[test]
    fn gpio_pins_try_into_u8_ok() {
        let gpio_num: u8 = (&GpioPins::GPIO_11).into();
        assert_eq!(gpio_num, 11);
    }
}