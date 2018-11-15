use rcc::{AHB2, CRRCR};
use stm32l4::stm32l4x2::RNG; //{rng, RNG};

/// Extension trait to activate the RNG
pub trait RngExt {

    /// Activates the RNG
    fn activate(self, ahb2: &mut AHB2, crrcr: &mut CRRCR) -> RNG;
    // TODO: this is not very useful
    fn is_enabled(&self) -> bool;

    // TODO: use the existing blocking/rng HAL trait
    // https://github.com/rust-embedded/embedded-hal/blob/master/src/blocking/rng.rs
    fn has_random_data(&self) -> bool;
    fn get_random_data(&self)-> u32;
}


impl RngExt for RNG {
    //type Parts = Parts;

    fn activate(self, ahb2: &mut AHB2, crrcr: &mut CRRCR) -> RNG {
        ahb2.enr().modify(|_, w| w.rngen().set_bit());
        crrcr.crrcr().modify(|_, w| w.hsi48on().set_bit()); // p. 180 in ref-manual
        self.cr.write(|w| w.rngen().bit(true));
        self
    }

    fn is_enabled(&self) -> bool {
        self.cr.read().rngen().bit()
    }

    fn has_random_data(&self) -> bool {
        self.sr.read().drdy().bit()
    }

    fn get_random_data(&self)-> u32 {
        while !self.has_random_data() {}
        self.dr.read().rndata().bits()
    }
}
