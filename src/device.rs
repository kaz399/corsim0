#[derive(Debug)]
pub struct DeviceMapping {
    pub adrs: u32,
    pub size: usize,
}

#[derive(Debug)]
pub struct MemoryMappedDevice {
    pub name: String,
    pub data: Box<[u8]>,
    pub mapping: DeviceMapping,
    pub readable: bool,
    pub writable: bool,
}

pub trait DeviceAccess {
    fn get_range(&self) -> DeviceMapping;
    fn set_range(&mut self, range: DeviceMapping);
    fn is_mapped(&self, pt: u32) -> bool {
        let mapping = self.get_range();
        if mapping.adrs <= pt && ((pt - mapping.adrs) as usize) < mapping.size {
            true
        } else {
            false
        }
    }

    fn read8(&self, adrs: u32) -> Option<u8>;
    fn read16(&self, adrs: u32) -> Option<u16>;
    fn read32(&self, adrs: u32) -> Option<u32>;

    fn write8(&mut self, adrs: u32, val: u8);
    fn write16(&mut self, adrs: u32, val: u16);
    fn write32(&mut self, adrs: u32, val: u32);
}

impl DeviceAccess for MemoryMappedDevice {
    fn get_range(&self) -> DeviceMapping {
        let ret_range: DeviceMapping = DeviceMapping { ..self.mapping };
        ret_range
    }

    fn set_range(&mut self, range: DeviceMapping) {
        self.mapping = range;
    }

    fn read8(&self, adrs: u32) -> Option<u8> {
        if self.readable && self.is_mapped(adrs) {
            let index: usize = (adrs - self.mapping.adrs) as usize;
            return Some(self.data[index]);
        }
        None
    }

    fn read16(&self, adrs: u32) -> Option<u16> {
        if self.readable {
            if self.is_mapped(adrs) && self.is_mapped(adrs + 1) {
                let index: usize = (adrs - self.mapping.adrs) as usize;
                let data16: u16 =
                    ((self.data[index + 1] as u16) << 8) | ((self.data[index]) as u16);
                return Some(data16);
            }
        }
        None
    }

    fn read32(&self, adrs: u32) -> Option<u32> {
        if self.readable {
            if self.is_mapped(adrs) && self.is_mapped(adrs + 3) {
                let index: usize = (adrs - self.mapping.adrs) as usize;
                let data32: u32 = ((self.data[index + 3] as u32) << 24)
                    | (((self.data[index + 2]) as u32) << 16)
                    | (((self.data[index + 1]) as u32) << 8)
                    | ((self.data[index]) as u32);
                return Some(data32);
            }
        }
        None
    }

    fn write8(&mut self, adrs: u32, val: u8) {
        if self.writable && self.is_mapped(adrs) {
            let index: usize = (adrs - self.mapping.adrs) as usize;
            self.data[index] = val;
        }
    }

    fn write16(&mut self, adrs: u32, val: u16) {
        if self.writable {
            if self.is_mapped(adrs) && self.is_mapped(adrs + 1) {
                let index: usize = (adrs - self.mapping.adrs) as usize;
                self.data[index] = (val & 0xff) as u8;
                self.data[index + 1] = ((val >> 8) & 0xff) as u8;
            }
        }
    }

    fn write32(&mut self, adrs: u32, val: u32) {
        if self.writable {
            if self.is_mapped(adrs) && self.is_mapped(adrs + 3) {
                let index: usize = (adrs - self.mapping.adrs) as usize;
                self.data[index] = (val & 0xff) as u8;
                self.data[index + 1] = ((val >> 8) & 0xff) as u8;
                self.data[index + 2] = ((val >> 16) & 0xff) as u8;
                self.data[index + 3] = ((val >> 24) & 0xff) as u8;
            }
        }
    }
}

pub struct SystemMap {
    pub map: Vec<MemoryMappedDevice>,
}

pub trait SystemMapAccess<'b> {
    fn get_device(&mut self, pt: u32) -> Option<&mut MemoryMappedDevice>;
    fn register_device(&mut self, dev: MemoryMappedDevice);

    fn read8(&mut self, adrs: u32) -> Result<u8, String>;
    fn read16(&mut self, adrs: u32) -> Result<u16, String>;
    fn read32(&mut self, adrs: u32) -> Result<u32, String>;

    fn write8(&mut self, adrs: u32, val: u8);
    fn write16(&mut self, adrs: u32, val: u16);
    fn write32(&mut self, adrs: u32, val: u32);
}

impl<'b> SystemMapAccess<'b> for SystemMap {
    fn get_device(&mut self, pt: u32) -> Option<&mut MemoryMappedDevice> {
        println!("adrs {:08x}:", pt);
        for dev in &mut self.map {
            print!(
                "  device: {} {:08x}-{:08x} ",
                dev.name,
                dev.mapping.adrs,
                dev.mapping.size - 1
            );
            if dev.is_mapped(pt) {
                println!("*");
                return Some(dev);
            } else {
                println!("");
            }
        }
        None
    }

    fn register_device(&mut self, dev: MemoryMappedDevice) {
        println!(
            "register device: {} {:08x} size:{:08x}",
            dev.name, dev.mapping.adrs, dev.mapping.size
        );
        self.map.push(dev);
    }

    fn read8(&mut self, adrs: u32) -> Result<u8, String> {
        match self.get_device(adrs) {
            Some(x) => match x.read8(adrs) {
                Some(data) => Ok(data),
                None => Err(format!(
                    "Error: read8(): can not access to {:08x} in {}",
                    adrs, x.name
                )),
            },
            None => Err(format!(
                "Error: read16(): no devices are assigned:{:08x}",
                adrs
            )),
        }
    }

    fn read16(&mut self, adrs: u32) -> Result<u16, String> {
        match self.get_device(adrs) {
            Some(x) => match x.read16(adrs) {
                Some(data) => Ok(data),
                None => Err(format!(
                    "Error: read16(): can not access to {:08x} in {}",
                    adrs, x.name
                )),
            },
            None => Err(format!(
                "Error: read16(): no devices are assigned:{:08x}",
                adrs
            )),
        }
    }

    fn read32(&mut self, adrs: u32) -> Result<u32, String> {
        match self.get_device(adrs) {
            Some(x) => match x.read32(adrs) {
                Some(data) => {
                    return Ok(data);
                }
                None => {
                    return Err(format!(
                        "Error: read32(): can not access to {:08x} in {}",
                        adrs, x.name
                    ));
                }
            },
            None => {
                return Err(format!(
                    "Error: read32(): no devices are assigned:{:08x}",
                    adrs
                ));
            }
        }
    }

    fn write8(&mut self, adrs: u32, val: u8) {
        match self.get_device(adrs) {
            Some(x) => x.write8(adrs, val),
            None => (),
        }
    }

    fn write16(&mut self, adrs: u32, val: u16) {
        match self.get_device(adrs) {
            Some(x) => x.write16(adrs, val),
            None => (),
        }
    }

    fn write32(&mut self, adrs: u32, val: u32) {
        match self.get_device(adrs) {
            Some(x) => x.write32(adrs, val),
            None => (),
        }
    }
}
