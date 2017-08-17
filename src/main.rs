// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify this file
// under the terms of the GNU General Public License, version 3, as published by
// the Free Software Foundation. You should have received a copy of the GNU
// General Public License along with Attalus. If not, see
// <http://www.gnu.org/licenses/>.

extern crate attalus;

use attalus::hardware::vdp::*;
use attalus::hardware::memory_map::*;
// use attalus::hardware::io::sms2::*;
use attalus::emulation_manager::*;

fn main() {
    let mut args: Vec<String> = Vec::new();
    args.extend(std::env::args());
    if args.len() < 3 {
        eprintln!("Usage: exec filename n");
        return;
    }
    let filename = &args[1];
    let smm = SegaMemoryMap::new_from_file(filename.clone()).unwrap();

    let mut em = EmulationManager::new(smm);

    let n: usize = args[2].parse().expect("Usage: exec filename n");

    let mut screen = NoScreen {};

    em.main_loop(&mut screen, n);
    // let mut win = attalus::sdl_wrap::video::WindowCanvas::new().unwrap();
    // win.set_window_size(700, 700);
    // win.set_title("Attalus");
    // win.set_logical_size(256, 192);

    // main_loop(&mut em, &mut win, n);

    // attalus::graphics::WindowCanvas::new(500, 500);
    // let sdl = sdl2::init().unwrap();
    // for driver in sdl2::render::drivers() {
    //     println!("{:?}", driver);
    // }
    // let video = sdl.video().unwrap();
    // let mut win = video.window(&"Attalus", 800, 800)
    //     .position(5, 5)
    //     .opengl()
    //     .resizable()
    //     .build().unwrap();
    // win.show();
    // let mut canvas = win.into_canvas().accelerated().build().unwrap();

    // canvas.set_draw_color(Color::RGB(0, 0, 0));
    // // fills the canvas with the color we set in `set_draw_color`.
    // canvas.clear();

    // // change the color of our drawing with a gold-color ...
    // canvas.set_draw_color(Color::RGB(255, 210, 0));
    // // A draw a rectangle which almost fills our window with it !
    // canvas.fill_rect(Rect::new(10, 10, 780, 580));

    // canvas.set_draw_color(Color::RGB(0, 210, 200));
    // canvas.draw_point((110, 100));
    // canvas.draw_point((111, 100));
    // canvas.draw_point((112, 100));
    // canvas.draw_point((113, 100));
    // canvas.draw_point((114, 100));
    // canvas.draw_point((115, 100));
    // canvas.draw_point((116, 10000));
    // // However the canvas has not been updated to the window yet,
    // // everything has been processed to an internal buffer,
    // // but if we want our buffer to be displayed on the window,
    // // we need to call `present`. We need to call this everytime
    // // we want to render a new frame on the window.
    // canvas.present();

    // let event = sdl.event().unwrap();
    // let mut ep = sdl.event_pump().unwrap();
    // for i in 0..400 {
    //     let ev = ep.poll_event();
    //     if ev.is_some() {
    //         println!("event {:?}", ev);
    //     }
    //     // event.flush_events(0, 0xFFFFFFFF);
    //     std::thread::sleep(std::time::Duration::from_millis(50));
    // }
    // let log = LogEverything::new(std::io::stdout());
    // let mut args: Vec<String> = Vec::new();
    // args.extend(std::env::args());
    // if args.len() < 3 {
    //     eprintln!("Usage: exec filename n");
    //     return;
    // }
    // let filename = &args[1];
    // let smmh =
    //     <SegaMemoryMapperHardware as MemoryMapperHardware>::
    //         new_from_file(filename.clone(), 0x2000).unwrap();

    // let mut em = EmulationManager::new(log, smmh);

    // let n: u32 = args[2].parse().expect("Usage: exec filename n");

    // main_loop(&mut em, n);
}
