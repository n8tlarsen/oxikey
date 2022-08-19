pub fn set_clocks(dev: &atsamd21j::Peripherals)
{
    unsafe
    {
        dev.SYSCTRL.dfllctrl.modify(|_r, w| {
            w.usbcrm().set_bit();
            w.ccdis().set_bit();
            w.mode().set_bit()
        });
        dev.SYSCTRL.dfllval.write(|w| w.bits(0xBB80_u32));
        dev.SYSCTRL.dfllctrl.modify(|_r, w| w.enable().set_bit());
        dev.GCLK.gendiv.write(|w| w.bits(0));
        dev.GCLK.genctrl.write(|w|
        {
            w.id().bits(0);
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
        w.gen().gclk0();
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
}

