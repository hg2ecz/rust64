// memory debug window
extern crate minifb;
use minifb::*;
use std::io::Write;
use c64;
use utils;
mod font;

const DEBUG_W: usize = 640;
const DEBUG_H: usize = 416;


pub struct Debugger
{
    debug_window: minifb::Window,
    raster_window: minifb::Window,
    font: font::SysFont,
    window_buffer: Vec<u32>,
    raster_buffer: Vec<u32>,
    mempage_offset: u32, // RAM preview memory page offset
    vic_display_state: bool,
    draw_mode: u8,
}

impl Debugger
{
    pub fn new() -> Debugger
    {
        Debugger {
            debug_window: Window::new("Debug window", DEBUG_W, DEBUG_H, Scale::X2).unwrap(),
            raster_window: Window::new("Raster", 63, 312, Scale::X1).unwrap(),
            font: font::SysFont::new(DEBUG_W, DEBUG_H),
            window_buffer: vec![0; DEBUG_W * DEBUG_H],
            raster_buffer: vec![0; 63 * 312],
            mempage_offset: 0,
            vic_display_state: false,
            draw_mode: 0,
        }
    }

    pub fn render(&mut self, cpu: &mut c64::cpu::CPUShared, memory: &mut c64::memory::MemShared)
    {
        self.draw_border();

        if self.debug_window.is_key_pressed(Key::End, KeyRepeat::No)
        {
            self.draw_mode += 1;
            if self.draw_mode > 3 { self.draw_mode = 0; }

            // clear memdump
            for y in 1..26
            {
                for x in 0..40
                {
                    self.clear_char(x, y);
                }
            }

            // clear hex region
            for y in 28..52
            {
                for x in 0..80
                {
                    self.clear_char(x, y);
                }
            }
        }

        match self.draw_mode {
            0 => self.draw_ram(memory),
            1 => self.draw_vic(memory),
            2 => self.draw_cia(cpu),
            3 => self.draw_color_ram(memory),
            _ => ()
        }

        self.draw_gfx_mode(memory);
        self.draw_data(memory);
        self.draw_cpu(cpu);

        self.debug_window.update(&self.window_buffer);
        self.raster_window.update(&self.raster_buffer);
    }

    fn draw_ram(&mut self, memory: &mut c64::memory::MemShared)
    {
        if self.debug_window.is_key_pressed(Key::PageUp, KeyRepeat::Yes)
        {
            self.mempage_offset += 0x400;
            if self.mempage_offset > 0xFC00 { self.mempage_offset = 0; }
        }

        if self.debug_window.is_key_pressed(Key::PageDown, KeyRepeat::Yes)
        {
            if self.mempage_offset == 0x0000 { self.mempage_offset = 0x10000; }
            self.mempage_offset -= 0x400;
        }
        
        // dump memory page to screen
        let mut start = 0x0000 + self.mempage_offset as u16;

        let mut title = Vec::new();
        let _ = write!(&mut title, "Memory page ${:04x}-${:04x}", start, start + 0x3FF);
        self.font.draw_text(&mut self.window_buffer, 0, 0, &String::from_utf8(title).unwrap().to_owned()[..], 0x0A);
        self.font.draw_text(&mut self.window_buffer, 34, 0, "*RAM*", 0x0E);

        let mut hex_offset_x = 0;

        for y in 0..25
        {
            for x in 0..40
            {
                let byte = memory.borrow_mut().get_ram_bank(c64::memory::MemType::RAM).read(start);
                self.font.draw_char(&mut self.window_buffer, 8*x as usize, 8 + 8*y as usize, byte, 0x05);

                self.draw_hex(hex_offset_x + x as usize, 27 + y as usize, byte);
                hex_offset_x += 1;
                start += 1;
            }

            hex_offset_x = 0;
        }
    }

    fn draw_vic(&mut self, memory: &mut c64::memory::MemShared)
    {
        let mut start = 0xD000;

        let mut title = Vec::new();
        let _ = write!(&mut title, "VIC ${:04x}-${:04x}", start, start + 0x03F);
        self.font.draw_text(&mut self.window_buffer, 0, 0, &String::from_utf8(title).unwrap().to_owned()[..], 0x0A);
        self.font.draw_text(&mut self.window_buffer, 34, 0, "*VIC*", 0x0E);
        
        let mut hex_offset_x = 0;

        for y in 0..25
        {
            for x in 0..40
            {
                let byte = memory.borrow_mut().get_ram_bank(c64::memory::MemType::IO).read(start);
                self.font.draw_char(&mut self.window_buffer, 8*x as usize, 8 + 8*y as usize, byte, 0x05);

                self.draw_hex(hex_offset_x + x as usize, 27 + y as usize, byte);
                hex_offset_x += 1;
                start += 1;

                if start == 0xD040 { return; }
            }

            hex_offset_x = 0;
        }
    }

    fn draw_cia(&mut self, cpu: &mut c64::cpu::CPUShared)
    {
        let mut start = 0xDC00;

        let mut title = Vec::new();
        let _ = write!(&mut title, "CIA ${:04x}-${:04x}", start, start + 0x1FF);
        self.font.draw_text(&mut self.window_buffer, 0, 0, &String::from_utf8(title).unwrap().to_owned()[..], 0x0A);
        self.font.draw_text(&mut self.window_buffer, 34, 0, "*CIA*", 0x0E);
        
        let mut hex_offset_x = 0;

        for y in 0..25
        {
            for x in 0..40
            {
                let byte = cpu.borrow_mut().read_byte(start);
                self.font.draw_char(&mut self.window_buffer, 8*x as usize, 8 + 8*y as usize, byte, 0x05);

                self.draw_hex(hex_offset_x + x as usize, 27 + y as usize, byte);
                hex_offset_x += 1;
                start += 1;

                if start == 0xDE00 { return; }
            }

            hex_offset_x = 0;
        }
    }

    fn draw_color_ram(&mut self, memory: &mut c64::memory::MemShared)
    {
        let mut start = 0xD800;

        let mut title = Vec::new();
        let _ = write!(&mut title, "COLOR ${:04x}-${:04x}", start, start + 0x3FF);
        self.font.draw_text(&mut self.window_buffer, 0, 0, &String::from_utf8(title).unwrap().to_owned()[..], 0x0A);
        self.font.draw_text(&mut self.window_buffer, 28, 0, "*COLOR RAM*", 0x0E);
        
        let mut hex_offset_x = 0;

        for y in 0..25
        {
            for x in 0..40
            {
                let byte = memory.borrow_mut().get_ram_bank(c64::memory::MemType::IO).read(start);
                self.font.draw_char(&mut self.window_buffer, 8*x as usize, 8 + 8*y as usize, byte, 0x05);

                self.draw_hex(hex_offset_x + x as usize, 27 + y as usize, byte);
                hex_offset_x += 1;
                start += 1;

                if start == 0xDC00 { return; }
            }

            hex_offset_x = 0;
        }
    }

    fn draw_hex(&mut self, x_pos: usize, y_pos: usize, byte: u8 )
    {
        let mut hex_value = Vec::new();
        let _ = write!(&mut hex_value, "{:02X}", byte);
        
        let mut base_color = utils::fetch_c64_color_rgba(byte >> 4);
        if base_color == 0 { base_color = 0x00333333; }
        //self.set_saturation(&mut base_color, (byte >> 4) as f64 / 15.0);
        
        // all black? make it at least somewhat visible
        if byte == 0 { base_color = 0x00101010; }
        
        self.font.draw_text_rgb(&mut self.window_buffer, x_pos, y_pos, &String::from_utf8(hex_value).unwrap().to_owned()[..], base_color);        
    }

    fn draw_data(&mut self, memory: &mut c64::memory::MemShared)
    {
        let d018 = memory.borrow_mut().get_ram_bank(c64::memory::MemType::IO).read(0xD018);
        let dd00 = memory.borrow_mut().get_ram_bank(c64::memory::MemType::IO).read(0xDD00);
        
        let mut vmatrix_txt = Vec::new();
        let mut char_txt = Vec::new();
        let mut bmp_txt = Vec::new();
        let mut bank_txt = Vec::new();
        let _ = write!(&mut vmatrix_txt, "${:04X}", 0x400 * ((d018 >> 4) & 0xF) as u16);
        let _ = write!(&mut char_txt, "${:04X}", 0x800 * ((d018 >> 1) & 0x07) as u16);
        let _ = write!(&mut bmp_txt, "${:04X}", 0x2000 * ((d018 >> 3) & 0x01) as u16);
        let _ = write!(&mut bank_txt, "${:04X}", 0xC000 - 0x4000 * (dd00 & 0x03) as u16);
        self.font.draw_text(&mut self.window_buffer, 43, 3, "Screen: ", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 51, 3, &String::from_utf8(vmatrix_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, 45, 4, "Char: ", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 51, 4, &String::from_utf8(char_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, 43, 5, "Bitmap: ", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 51, 5, &String::from_utf8(bmp_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, 41, 6, "VIC Bank: ", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 51, 6, &String::from_utf8(bank_txt).unwrap().to_owned()[..], 0x0E);
    }

    fn draw_gfx_mode(&mut self, memory: &mut c64::memory::MemShared)
    {
        let d011 = memory.borrow_mut().get_ram_bank(c64::memory::MemType::IO).read(0xD011);
        let d016 = memory.borrow_mut().get_ram_bank(c64::memory::MemType::IO).read(0xD016);
        let ecm_on = (d011 & 0x40) != 0;
        let mcm_on = (d016 & 0x10) != 0;
        let bmp_on = (d011 & 0x20) != 0;
        
        self.font.draw_text(&mut self.window_buffer, 52, 1, "ECM", if ecm_on { 0x0A } else { 0x0B });
        self.font.draw_text(&mut self.window_buffer, 57, 1, "CHR", if !bmp_on & !ecm_on { 0x0A } else { 0x0B });
        self.font.draw_text(&mut self.window_buffer, 62, 1, "BMP", if bmp_on { 0x0A } else { 0x0B });
        self.font.draw_text(&mut self.window_buffer, 67, 1, "MCM", if mcm_on { 0x0A } else { 0x0B });
    }

    fn draw_cpu(&mut self, cpu: &mut c64::cpu::CPUShared)
    {
        let mut pc_txt = Vec::new();
        let mut a_txt = Vec::new();
        let mut x_txt = Vec::new();
        let mut y_txt = Vec::new();
        let mut sp_txt = Vec::new();
        let mut p_txt = Vec::new();
        let _ = write!(&mut pc_txt, "${:04X}", cpu.borrow_mut().PC);
        let _ = write!(&mut a_txt, "${:02X}", cpu.borrow_mut().A);
        let _ = write!(&mut x_txt, "${:02X}", cpu.borrow_mut().X);
        let _ = write!(&mut y_txt, "${:02X}", cpu.borrow_mut().Y);
        let _ = write!(&mut sp_txt, "${:02X}", cpu.borrow_mut().SP);
        let _ = write!(&mut p_txt, "[{:08b}]", cpu.borrow_mut().P);
        
        self.font.draw_text(&mut self.window_buffer, 44, 23, "PC:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 47, 23, &String::from_utf8(pc_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, 53, 23, "A:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 55, 23, &String::from_utf8(a_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, 59, 23, "X:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 61, 23, &String::from_utf8(x_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, 65, 23, "Y:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 67, 23, &String::from_utf8(y_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, 71, 23, "SP:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 74, 23, &String::from_utf8(sp_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, 51, 24, "NV-BDIZC:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, 61, 24, &String::from_utf8(p_txt).unwrap().to_owned()[..], 0x0E);
    }

    fn draw_border(&mut self)
    {
        for x in 0..80
        {
            self.font.draw_char(&mut self.window_buffer, 8*x as usize, 0, 64, 0x0B);
            self.font.draw_char(&mut self.window_buffer, 8*x as usize, 8*26, 64, 0x0B);
        }
        
        for y in 1..26
        {
            self.font.draw_char(&mut self.window_buffer, 8*40, 8*y as usize, 66, 0x0B);
        }
        
        self.font.draw_char(&mut self.window_buffer, 8*40, 0, 114, 0x0B);
        self.font.draw_char(&mut self.window_buffer, 8*40, 8*26, 113, 0x0B);
    }

    pub fn update_raster_window(&mut self, vic: &mut c64::vic::VICShared)
    {
        let x = vic.borrow_mut().curr_cycle as u16;
        let y = vic.borrow_mut().raster_cnt;
        let is_bad_line = vic.borrow_mut().is_bad_line;
        let is_raster_irq = vic.borrow_mut().raster_irq == y;
        let is_border = vic.borrow_mut().border_on;
        let is_state_changed = self.vic_display_state != vic.borrow_mut().dbg_reg_written;
        self.vic_display_state = vic.borrow_mut().dbg_reg_written;
        let border_color = 0x00404040;
        let bg_color = 0x00000000;
        let state_color = 0x00FF0000;
        let raster_color = 0x000000FF;
        let badline_color = 0x0000FF00;
        
        let mut dst_color = if is_border { border_color } else { bg_color };
        dst_color = if is_state_changed { self.mix_colors(state_color, dst_color, 0.8) } else { dst_color };
        dst_color = if is_raster_irq { self.mix_colors(raster_color, dst_color, 0.8) } else { dst_color };
        dst_color = if is_bad_line { self.mix_colors(badline_color, dst_color, 0.5) } else { dst_color };
        
        self.raster_buffer[(x - 1 + y * 63) as usize] = dst_color
    }
    
    fn clear_char(&mut self, x_pos: usize, y_pos: usize)
    {
        self.font.draw_text(&mut self.window_buffer, x_pos, y_pos, " ", 0x00);
    }


    fn mix_colors(&self, new: u32, old: u32, alpha: f32) -> u32
    {
        let rn = ((new >> 16) & 0xFF) as f32;
        let gn = ((new >> 8) & 0xFF) as f32;
        let bn = (new & 0xFF) as f32;

        let ro = ((old >> 16) & 0xFF) as f32;
        let go = ((old >> 8) & 0xFF) as f32;
        let bo = (old & 0xFF) as f32;

        let rd = alpha * rn + (1.0 - alpha) * ro;
        let gd = alpha * gn + (1.0 - alpha) * go;
        let bd = alpha * bn + (1.0 - alpha) * bo;

        let mut dst_color = (rd as u32) << 16;
        dst_color |= (gd as u32) << 8;
        dst_color |= bd as u32;

        dst_color
    }
    
   /* fn set_saturation(&self, color: &mut u32, change: f64)
    {
        let Pr = 0.299;
        let Pg = 0.587;
        let Pb = 0.114;
        let mut r = ((*color >> 16) & 0xFF) as f64;
        let mut g = ((*color >> 8) & 0xFF) as f64;
        let mut b = (*color & 0xFF) as f64;

        let P = ((r*r*Pr + g*g*Pg + b*b*Pb) as f64).sqrt();

        r = P + (r - P) * change;
        g = P + (g - P) * change;
        b = P + (b - P) * change;

        *color = (r as u32) << 16;
        *color |= (g as u32) << 8;
        *color |= b as u32;
    }*/
}