use crate::emulator::Emulator;
use crate::memory::MemoryBus;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub(crate) struct IPU {
    mem: &MemoryBus,
    pub poll: EventLoop,
}

impl IPU {
    pub fn new(em: &Emulator) -> Self{
        let poll = EventLoop::new();
        poll.run(
            Event::WindowEvent { event, em.dsp }
        )

        IPU {
            mem: &em.mem,
            poll: EventLoop::new(),
        }        
    }
}