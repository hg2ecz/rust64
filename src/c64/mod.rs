extern crate sdl2;
pub mod cpu;
pub mod opcodes;
mod clock;
mod memory;
mod vic;

pub struct C64
{
    memory: memory::MemShared,
    clock: clock::Clock,
    cpu: cpu::CPU,
    vic: vic::VIC,
}

impl C64
{
    pub fn new(renderer: &sdl2::render::Renderer) -> C64
    {
        let memory : memory::MemShared = memory::Memory::new_shared();
        
        C64
        {
            memory: memory.clone(),                     // shared system memory (RAM, ROM, IO registers)
            clock: clock::Clock::new(),
            cpu: cpu::CPU::new(memory.clone()),
            vic: vic::VIC::new(memory.clone(), renderer),
        }
    }

    pub fn reset(&mut self)
    {
        self.memory.borrow_mut().reset();
        self.cpu.reset();
    }
    
    
    pub fn update(&mut self)
    {
        if self.clock.tick() { println!("Clock tick"); }
        self.cpu.update();
    }


    pub fn render(&self, renderer: &mut sdl2::render::Renderer)
    {
        self.vic.render(renderer);
    }
}
