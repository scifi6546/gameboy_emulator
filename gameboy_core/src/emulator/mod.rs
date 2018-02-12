pub mod traits;

use cpu::Cpu;
use gpu::GPU;
use mmu::Memory;
use mmu::interrupt::Interrupt;
use self::traits::Io;
use joypad::Joypad;

pub struct Emulator {
    cpu: Cpu,
    gpu: GPU,
    memory: Memory,
    joypad: Joypad,
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            cpu: Cpu::new(),
            gpu: GPU::new(),
            memory: Memory::new(),
            joypad: Joypad::new(),
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.memory.load_rom(rom);
    }

    pub fn update<T: Io>(&mut self, io: &mut T) -> &[u8; 144 * 160] {
        let max_cycles = 69905;
        let mut cycles_this_update = 0;

        while cycles_this_update < max_cycles {
            let cycles = if !self.cpu.stopped && !self.cpu.halted {
                self.cpu.step(&mut self.memory)
            } else {
                1
            };
            cycles_this_update += cycles;
            self.gpu.step(cycles, &mut self.memory);
            self.handle_input(io);
            self.handle_interrupts();
        }

        &self.gpu.pixels
    }

    fn handle_input<T: Io>(&mut self, controller: &mut T) {
        controller.update_joypad(&mut self.joypad);
        self.joypad.save_to_memory(&mut self.memory);
    }

    fn handle_interrupts(&mut self) {
        if self.cpu.interrupt_enabled {
            if let Some(interrupt) = self.memory.get_interrupt() {
                self.process_interrupt(interrupt);
            }
        }
    }

    fn process_interrupt(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::Vblank => self.cpu.rst_40(&mut self.memory),
            Interrupt::Lcd => self.cpu.rst_48(&mut self.memory),
            Interrupt::Timer => self.cpu.rst_50(&mut self.memory),
            Interrupt::Serial => self.cpu.rst_58(&mut self.memory),
            Interrupt::Joypad => self.cpu.rst_60(&mut self.memory),
        }

        self.memory.remove_interrupt(interrupt);
        self.cpu.halted = false;
    }
}
