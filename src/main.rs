use std::boxed::Box;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::exit;

mod cpu;
mod device;

use crate::cpu::SystemCtrl;
use crate::device::SystemMapAccess;

const ROMADDR: u32 = 0x00000000;
const ROMSIZE: usize = 128 * 1024;

const RAMADDR: u32 = 0x10000000;
const RAMSIZE: usize = 128 * 1024;

fn main() {
    let args: Vec<String> = env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        println!("Usage: corsim0 image-file");
        exit(1);
    }

    let filename: String = args[1].clone();

    let mut rom: device::MemoryMappedDevice = device::MemoryMappedDevice {
        name: "ROM".to_string(),
        data: Box::new([0; RAMSIZE]),
        mapping: device::DeviceMapping {
            adrs: ROMADDR,
            size: ROMSIZE,
        },
        readable: true,
        writable: false,
    };

    let ram: device::MemoryMappedDevice = device::MemoryMappedDevice {
        name: "RAM".to_string(),
        data: Box::new([0; RAMSIZE]),
        mapping: device::DeviceMapping {
            adrs: RAMADDR,
            size: RAMSIZE,
        },
        readable: true,
        writable: true,
    };

    match File::open(filename) {
        Ok(mut f) => {
            // Load ROM image
            match f.read(&mut rom.data) {
                Ok(readed) => println!("readed: {}", readed),
                Err(e) => println!("errro: {}", e),
            }

            let mut device_map :device::SystemMap = device::SystemMap { map: Vec::new() };
            device_map.register_device(ram);
            device_map.register_device(rom);

            let mut system: cpu::M0System = cpu::M0System::new(device_map);

            println!("reset vector {}", system.system_map.read32(0).unwrap());
            let mut cycle_count: u32 = 1;

            system.reset();
            system.dump();

            println!("*EXECUTE BINARY");
            loop {
                println!("");
                print!("clk:{}\t", cycle_count);
                let elapsed_cycle: u32 = system.execute();
                if elapsed_cycle > 0 {
                    cycle_count += elapsed_cycle;
                } else {
                    println!("");
                    println!("*FATAL ERROR (EXIT)");
                    break;
                }
            }
        }
        Err(e) => println!("error {} {}", args[1], e),
    }
}
