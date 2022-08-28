use cortex_m_semihosting::hprintln;
use bitfield::bitfield;

bitfield!{
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

pub fn setup(dev: &atsamd21j::Peripherals)
{
    unsafe
    {
        let cal = & *(0x0080_6020_u32 as *const Calibration);
        hprintln!("dfll48m calibration value: {}", cal.dfll48m_coarse_cal()).unwrap();
        dev.SYSCTRL.dfllctrl.write(|w| w.bits(0x0001_u16));
        while dev.SYSCTRL.pclksr.read().dfllrdy().bit_is_clear() {}
        dev.SYSCTRL.dfllval.write(|w| w.coarse().bits(cal.dfll48m_coarse_cal() as u8));
        while dev.SYSCTRL.pclksr.read().dfllrdy().bit_is_clear() {}
        dev.SYSCTRL.dfllmul.write(|w| w.mul().bits(0xBB80_u16));
        while dev.SYSCTRL.pclksr.read().dfllrdy().bit_is_clear() {}
        dev.SYSCTRL.dfllctrl.write(|w| 
        {
            w.usbcrm().set_bit();
            w.ccdis().set_bit();
            w.mode().set_bit();
            w.enable().set_bit()
        });
        dev.GCLK.gendiv.write(|w| w.bits(1));
        dev.GCLK.genctrl.write(|w|
        {
            w.id().bits(1);
            w.src().dfll48m();
            w.genen().set_bit();
            w.idc().clear_bit();
            w.oov().clear_bit();
            w.oe().clear_bit();
            w.divsel().clear_bit();
            w.runstdby().set_bit()
        }); 
    }
    dev.GCLK.clkctrl.write(|w| 
    {
        w.id().usb();
        w.gen().gclk1();
        w.clken().set_bit();
        w.wrtlock().set_bit()
    });
    dev.GCLK.clkctrl.write(|w| 
    {
        w.id().tcc0_tcc1();
        w.gen().gclk0();
        w.clken().set_bit();
        w.wrtlock().set_bit()
    });
    dev.GCLK.clkctrl.write(|w| 
    {
        w.id().tcc2_tc3();
        w.gen().gclk0();
        w.clken().set_bit();
        w.wrtlock().set_bit()
    });
    dev.GCLK.clkctrl.write(|w| 
    {
        w.id().eic();
        w.gen().gclk0();
        w.clken().set_bit();
        w.wrtlock().set_bit()
    });
    dev.PM.apbcmask.write(|w| w.tcc0_().set_bit());
    unsafe { dev.PORT.dirset1.write(|w| w.bits(0x4000_0000_u32)); }
    dev.PORT.pmux1_[15].write(|w| w.pmuxe().e());
    dev.PORT.pincfg1_[30].write(|w| w.pmuxen().set_bit());
    unsafe{ dev.TCC0.per().write(|w| w.per().bits(0x0008_0000_u32)); }
    // dev.TCC0.intenset.write(|w| w.ovf().set_bit());
    dev.TCC0.ctrla.write(|w|
    {
        w.enable().set_bit();
        w.runstdby().set_bit()
    });
}

