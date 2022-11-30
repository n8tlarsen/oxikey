use bitfield::bitfield;

bitfield! {
    pub struct Calibration(u128);
    impl Debug;
    u32;
        reserved0,          _ :   2,  0;
        reserved1,          _ :  14,  3;
        reserved2,          _ :  26, 15;
    pub adc_linearity,      _ :  34, 27;
    pub adc_biascal,        _ :  37, 35;
    pub osc32k_cal,         _ :  44, 38;
    pub usb_transn,         _ :  49, 45;
    pub usb_transp,         _ :  54, 50;
    pub usb_trim,           _ :  57, 55;
    pub dfll48m_coarse_cal, _ :  63, 58;
        reserved3,          _ :  73, 64;
        reserved4,          _ : 127, 74;
}

pub fn blink(pm: &atsamd21g::PM, port: &atsamd21g::PORT, tcc0: &atsamd21g::TCC0) {
    // start timer in PWM mode to flash LED
    pm.apbcmask.write(|w| w.tcc0_().set_bit());
    unsafe {
        port.dirset1.write(|w| w.bits(0x4000_0000_u32));
    }
    port.pmux1_[15].write(|w| w.pmuxe().e());
    port.pincfg1_[30].write(|w| w.pmuxen().set_bit());
    unsafe {
        tcc0.per().write(|w| w.per().bits(48000000_u32));
    }
    // dev.TCC0.intenset.write(|w| w.ovf().set_bit());
    tcc0.ctrla.write(|w| {
        w.enable().set_bit();
        w.runstdby().set_bit()
    });
}
