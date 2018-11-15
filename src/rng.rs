use rcc::{AHB2, CRRCR};
use stm32l4::stm32l4x2::RNG; //{rng, RNG};

/// Extension trait to activate the RNG
pub trait RngExt {

    /// Activates the RNG
    fn activate(self, ahb2: &mut AHB2, crrcr: &mut CRRCR) -> RNG;

    fn has_random_data(&self) -> bool;
}


impl RngExt for RNG {
    //type Parts = Parts;

    fn activate(self, ahb2: &mut AHB2, crrcr: &mut CRRCR) -> RNG {
        ahb2.enr().modify(|_, w| w.rngen().set_bit());
        crrcr.crrcr().modify(|_, w| w.hsi48on().set_bit()); // p. 180 in ref-manual
        self.cr.write(|w| w.rngen().bit(true));
        self
    }

    fn has_random_data(&self) -> bool {
        self.sr.read().drdy().bit()
    }
}
