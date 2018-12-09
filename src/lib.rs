mod cpu;
mod device;

use crate::cpu::SystemCtrl;
use crate::device::SystemMapAccess;

const ROMADDR: u32 = 0x00000000;
const ROMSIZE: usize = 128;

const RAMADDR: u32 = 0x10000000;
const RAMSIZE: usize = 128;

#[test]
fn ram_write_read() {
    let mut ram: device::MemoryMappedDevice = device::MemoryMappedDevice {
        name: "RAM".to_string(),
        data: Box::new([0; RAMSIZE]),
        mapping: device::DeviceMapping {
            adrs: RAMADDR,
            size: RAMSIZE,
        },
        readable: true,
        writable: true,
    };

    let mut system_map: device::SystemMap = device::SystemMap { map: Vec::new() };

    system_map.register_device(ram);

    let mut write_val: u8 = 0;
    for i in 0..RAMSIZE {
        let adrs: u32 = RAMADDR + (i as u32);
        system_map.write8(adrs, write_val);
        let read_val = system_map.read8(adrs).unwrap();
        assert_eq!(write_val, read_val);
        if write_val == 0xfe {
            write_val = 0;
        }
        else {
            write_val += 1;
        }
    }
}

#[test]
fn rom_write() {
    let mut rom: device::MemoryMappedDevice = device::MemoryMappedDevice {
        name: "ROM".to_string(),
        data: Box::new([0; ROMSIZE]),
        mapping: device::DeviceMapping {
            adrs: ROMADDR,
            size: ROMSIZE,
        },
        readable: true,
        writable: false,
    };

    let mut system_map: device::SystemMap = device::SystemMap { map: Vec::new() };

    system_map.register_device(rom);

    for i in 0..ROMSIZE {
        let adrs: u32 = ROMADDR + (i as u32);
        let write_val: u8 = (i / 2) as u8;
        let rom_val = system_map.read8(adrs).unwrap();
        system_map.write8(adrs, rom_val + 1);
        let rom_val2 = system_map.read8(adrs).unwrap();
        // ROM is unwritable 
        assert_eq!(rom_val, rom_val2);
    }
}

