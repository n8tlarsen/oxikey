pub fn set_system(sys: &atsamd21j::SYSCTRL)
{
    sys.dfllctrl.modify(|_r, w| {
        w.usbcrm().set_bit();
        w.ccdis().set_bit();
        w.mode().set_bit()
    });
    unsafe{ sys.dfllval.write(|w| w.bits(0xBB80_u32)); }
    sys.dfllctrl.modify(|_r, w| w.enable().set_bit());
}

pub fn set_clocks(clk: &atsamd21j::GCLK)
{
    unsafe
    {
        clk.gendiv.write(|w| w.bits(0));
        clk.genctrl.write(|w|
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
        clk.clkctrl.write(|w| 
        {
            w.id().usb();
            w.gen().gclk0();
            w.clken().set_bit();
            w.wrtlock().set_bit()
        });
        clk.clkctrl.write(|w| 
        {
            w.id().tcc0_tcc1();
            w.gen().gclk0();
            w.clken().set_bit();
            w.wrtlock().set_bit()
        });
        clk.clkctrl.write(|w| 
        {
            w.id().tcc2_tc3();
            w.gen().gclk0();
            w.clken().set_bit();
            w.wrtlock().set_bit()
        });
        clk.clkctrl.write(|w| 
        {
            w.id().eic();
            w.gen().gclk0();
            w.clken().set_bit();
            w.wrtlock().set_bit()
        });
    }
}
