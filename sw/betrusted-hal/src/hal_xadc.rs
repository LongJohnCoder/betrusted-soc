
pub enum XadcRegs {
    Temperature = 0x0,
    VccInt = 0x1,
    VccAux = 0x2,
    Dedicated = 0x3,
    VrefP = 4,
    VrefN = 5,
    VccBram = 6,

    Vaux0 = 16,
    Vaux1 = 17,
    Vaux2 = 18,
    Vaux3 = 19,
    Vaux4 = 20,
    Vaux5 = 21,
    Vaux6 = 22,
    Vaux7 = 23,
    Vaux8 = 24,
    Vaux9 = 25,
    Vaux10 = 26,
    Vaux11 = 27,
    Vaux12 = 28,
    Vaux13 = 29,
    Vaux14 = 30,
    Vaux15 = 31,

    // useful to detect glitches
    MaxTemp = 0x20,
    MaxVccInt = 0x21,
    MaxVccAux = 0x22,
    MaxVccBram = 0x23,
    MinTemp = 0x24,
    MinVccInt = 0x25,
    MinVccAux = 0x26,
    MinVccBram = 0x27,

    Config0 = 0x40,
    Config1 = 0x41,
    Config2 = 0x42,
    Seq0 = 0x48,
    Seq1 = 0x49,
    SeqAvg0 = 0x4A,
    SeqAvg1 = 0x4B,
    SeqMode0 = 0x4C,
    SeqMode1 = 0x4D,
    SeqSettling0 = 0x4E,
    SeqSettling1 = 0x4F,

    // auto-alarm thresholds. Alarms allow real time monitoring of supplies without processor intervention.
    AlarmTempUpper = 0x50,
    AlarmVccIntUpper = 0x51,
    AlarmVccAuxUpper = 0x52,
    AlarmOtLimit = 0x53,
    AlarmTempLower = 0x54,
    AlarmVccIntLower = 0x55,
    AlarmVccAuxLower = 0x56,
    AlarmOtReset = 0x57,
    AlarmVccBramUpper = 0x58,

    AlarmVccBramLower = 0x5C,
}

#[allow(dead_code)]
fn xadc_write(p: &betrusted_pac::Peripherals, adr: XadcRegs, data: u16) {
    unsafe{ p.INFO.xadc_drp_adr.write(|w| w.bits(adr as u32)); }
    unsafe{ p.INFO.xadc_drp_dat_w1.write(|w| w.bits(((data >> 8) & 0xff) as u32)); }
    unsafe{ p.INFO.xadc_drp_dat_w0.write(|w| w.bits((data & 0xff) as u32)); }
    unsafe{ p.INFO.xadc_drp_write.write(|w| w.bits(1)); } // commit the write

    while p.INFO.xadc_drp_drdy.read().bits() == 0 {} // wait for the write to complete
}

#[allow(dead_code)]
fn xadc_read(p: &betrusted_pac::Peripherals, adr: XadcRegs) -> u16 {
    let mut ret: u16;
    unsafe{ p.INFO.xadc_drp_adr.write(|w| w.bits(adr as u32)); }
    unsafe{ p.INFO.xadc_drp_read.write(|w| w.bits(1)); } // trigger the read

    while p.INFO.xadc_drp_drdy.read().bits() == 0 {} // wait for the read to complete

    ret = p.INFO.xadc_drp_dat_r0.read().bits() as u16 & 0xFF;
    ret = (((p.INFO.xadc_drp_dat_r1.read().bits() as u16) & 0xFF) << 8) | ret;

    ret
}

#[allow(dead_code)]
fn xadc_enable(p: &betrusted_pac::Peripherals, enable: bool) {
    if enable {
        unsafe{ p.INFO.xadc_drp_enable.write(|w| w.bits(1)); } 
    } else {
        unsafe{ p.INFO.xadc_drp_enable.write(|w| w.bits(0)); } 
    }
}

pub enum XadcSeq {
    Default = 0,
    SinglePass = 1,
    Continuous = 2,
    SingleChannel = 3,
    Simultaneous = 4,
    Independent = 8,
}

pub enum XadcPower {
    AllOn = 0,
    AdcbOff = 2,
    AllOff = 3,
}

pub enum XadcChannel {
    Temperature = 0,
    VccInt = 1,
    VccAux = 2,
    Dedicated = 3,
    VrefP = 4,
    VrefN = 5,
    VccBram = 6,
    Calibrate = 8,
    Vaux0 = 16,
    Vaux1 = 17,
    Vaux2 = 18,
    Vaux3 = 19,
    Vaux4 = 20,
    Vaux5 = 21,
    Vaux6 = 22,
    Vaux7 = 23,
    Vaux8 = 24,
    Vaux9 = 25,
    Vaux10 = 26,
    Vaux11 = 27,
    Vaux12 = 28,
    Vaux13 = 29,
    Vaux14 = 30,
    Vaux15 = 31,
}

pub enum XadcFilter {
    None = 0,
    Avg16 = 1,
    Avg64 = 2,
    Avg256 = 3,
}

pub enum XadcCalBitMask {
    AdcOffset = 1,
    AdcOffsetAndGain = 2,
    SupplyOffset = 3,
    SupplyOffsetAndGain = 4,
}

pub enum BtXadcMode {
    RoundRobin, // round robin sampling of active ports
    Stream,     // streaming of just one channel -- TODO
}

pub struct BtXadc {
    p: betrusted_pac::Peripherals,
    mode: BtXadcMode,
}

impl BtXadc {
    pub fn new() -> Self {
        let ret: BtXadc;
        unsafe {
            ret = BtXadc {
                p: betrusted_pac::Peripherals::steal(),
                mode: BtXadcMode::RoundRobin,
            };
        }
        xadc_enable(&ret.p, true);
        
        // 0x8000 is constant -- disables averaging of cal bit
        xadc_write(&ret.p, XadcRegs::Config0, 0x8000 | (XadcFilter::Avg16 as u16) << 12);
        // 0x0EF0 is constant -- disables alarms not present on this chip, enables calibration, enables all other alarms
        xadc_write(&ret.p, XadcRegs::Config1, ((XadcSeq::Continuous as u16) << 12) | 0x0EF0 );
        // 0x0400 is constant -- sets DCLK to SYSCLK/4 = 25MHz
        xadc_write(&ret.p, XadcRegs::Config2, 0x0400 | (XadcPower::AllOn as u16) << 4); 

        xadc_write(&ret.p, XadcRegs::Seq0, 0x4F01); // selects VCCBRAM, dedicated, VCCAUX, VCCINT, temp, cal
        xadc_write(&ret.p, XadcRegs::Seq1, 0x0F01); // selects aux channels 11, 10, 9, 8, and 0

        xadc_write(&ret.p, XadcRegs::SeqAvg0, 0x4F00); // average VCCBRAM, dedicated, VCCAUX, VCCINT
        xadc_write(&ret.p, XadcRegs::SeqAvg1, 0x0E00); // average only channels 9, 10, 11 (not the noise channels)

        ret
    }

    pub fn set_mode(&mut self, mode: BtXadcMode) {
        self.mode = mode;
    }

    /// blocks until the latest sequence finishes, guarantees the values have been updated
    pub fn wait_update(&mut self) {
        while self.p.INFO.xadc_eos.read().bits() == 0 {}
    }

    pub fn noise0(&mut self) -> u16 {
        xadc_read(&self.p, XadcRegs::Vaux0) >> 4
    }
    pub fn noise1(&mut self) -> u16 {
        xadc_read(&self.p, XadcRegs::Vaux8) >> 4
    }
    pub fn vbus_mv(&mut self) -> u16 {
        // voltage is 0.0485 * VBUS
        // ADC code is 1/4096 of a volt
        let code: u32 = xadc_read(&self.p, XadcRegs::Vaux9) as u32 >> 4;

        // e.g., code of 993 = 5V will return 4997mV
        ((code * 5033) / 1000) as u16
    }
    pub fn cc1_mv(&mut self) -> u16 {
        let code: u32 = xadc_read(&self.p, XadcRegs::Vaux10) as u32 >> 4;

        // voltage is 1.0 * CC level (safely saturates due to HW protection above 1.0V)
        // ADC code is 1/4096 of a volt
        (code * 1000 / 4096) as u16
    }
    pub fn cc2_mv(&mut self) -> u16 {
        let code: u32 = xadc_read(&self.p, XadcRegs::Vaux11) as u32 >> 4;

        // voltage is 1.0 * CC level (safely saturates due to HW protection above 1.0V)
        // ADC code is 1/4096 of a volt
        (code * 1000 / 4096) as u16
    }
    pub fn vccint(&mut self) -> u16 { xadc_read(&self.p, XadcRegs::VccInt) >> 4 }
    pub fn vccaux(&mut self) -> u16 { xadc_read(&self.p, XadcRegs::VccAux) >> 4 }
    pub fn vccbram(&mut self) -> u16 { xadc_read(&self.p, XadcRegs::VccBram) >> 4 }
    pub fn temp(&mut self) -> u16 { xadc_read(&self.p, XadcRegs::Temperature) >> 4 }
    pub fn audio_sample(&mut self) -> u16 { xadc_read(&self.p, XadcRegs::Dedicated) >> 4 }

}