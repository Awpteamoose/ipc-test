use miow::pipe::{NamedPipe, connect};
use std::io::{Read, Write};
use std::mem::size_of;
use std::sync::{RwLock, Arc};

#[repr(C)]
#[derive(Default, Debug)]
#[allow(non_snake_case)]
struct Status {
	TrackLeft: bool,
	TrackRight: bool,

	Cycle: bool,

	MarkerSet: bool,
	MarkerLeft: bool,
	MarkerRight: bool,

	FastLeft: bool,
	FastRight: bool,

	Stop: bool,
	Play: bool,
	Record: bool,

	Fader0: u8,
	Fader1: u8,
	Fader2: u8,
	Fader3: u8,
	Fader4: u8,
	Fader5: u8,
	Fader6: u8,
	Fader7: u8,

	Knob0: u8,
	Knob1: u8,
	Knob2: u8,
	Knob3: u8,
	Knob4: u8,
	Knob5: u8,
	Knob6: u8,
	Knob7: u8,

	S0: bool,
	S1: bool,
	S2: bool,
	S3: bool,
	S4: bool,
	S5: bool,
	S6: bool,
	S7: bool,

	M0: bool,
	M1: bool,
	M2: bool,
	M3: bool,
	M4: bool,
	M5: bool,
	M6: bool,
	M7: bool,

	R0: bool,
	R1: bool,
	R2: bool,
	R3: bool,
	R4: bool,
	R5: bool,
	R6: bool,
	R7: bool,
}

impl Status {
	fn button(&mut self, id: u8, status: bool) {
		match id {
			0x3A => self.TrackLeft = status,
			0x3B => self.TrackRight = status,

			0x2E => self.Cycle = status,

			0x3C => self.MarkerSet = status,
			0x3D => self.MarkerLeft = status,
			0x3E => self.MarkerRight = status,

			0x2B => self.FastLeft = status,
			0x2C => self.FastRight = status,

			0x2A => self.Stop = status,
			0x29 => self.Play = status,
			0x2D => self.Record = status,

			0x20 => self.S0 = status,
			0x21 => self.S1 = status,
			0x22 => self.S2 = status,
			0x23 => self.S3 = status,
			0x24 => self.S4 = status,
			0x25 => self.S5 = status,
			0x26 => self.S6 = status,
			0x27 => self.S7 = status,

			0x30 => self.M0 = status,
			0x31 => self.M1 = status,
			0x32 => self.M2 = status,
			0x33 => self.M3 = status,
			0x34 => self.M4 = status,
			0x35 => self.M5 = status,
			0x36 => self.M6 = status,
			0x37 => self.M7 = status,

			0x40 => self.R0 = status,
			0x41 => self.R1 = status,
			0x42 => self.R2 = status,
			0x43 => self.R3 = status,
			0x44 => self.R4 = status,
			0x45 => self.R5 = status,
			0x46 => self.R6 = status,
			0x47 => self.R7 = status,
			_ => (),
		}
	}

	fn analog(&mut self, id: u8, value: u8) {
		match id {
			0x00 => self.Fader0 = value,
			0x01 => self.Fader1 = value,
			0x02 => self.Fader2 = value,
			0x03 => self.Fader3 = value,
			0x04 => self.Fader4 = value,
			0x05 => self.Fader5 = value,
			0x06 => self.Fader6 = value,
			0x07 => self.Fader7 = value,

			0x10 => self.Knob0 = value,
			0x11 => self.Knob1 = value,
			0x12 => self.Knob2 = value,
			0x13 => self.Knob3 = value,
			0x14 => self.Knob4 = value,
			0x15 => self.Knob5 = value,
			0x16 => self.Knob6 = value,
			0x17 => self.Knob7 = value,
			_ => (),
		}
	}
}

impl std::ops::Deref for Status {
	type Target = [u8; size_of::<Self>()];

	fn deref(&self) -> &Self::Target {
		unsafe { &*(self as *const Self as *mut [u8; size_of::<Self>()]) }
	}
}

impl std::ops::DerefMut for Status {
	type Target = [u8; size_of::<Self>()];

	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *(self as *mut Self as *mut [u8; size_of::<Self>()]) }
	}
}

fn main() {
	for argument in std::env::args() {
		if argument == "server" {
			let mut midi_in = midir::MidiInput::new("nanokontrol").unwrap();
			midi_in.ignore(midir::Ignore::None);

			let status_arc = Arc::new(RwLock::new(Status::default()));

			let status = Arc::clone(&status_arc);
			let _conn_in = midi_in.connect(0, "nanokontrol-huh", move |stamp, message, _| {
				let mut status = status.write().unwrap();

				match message {
					[144, button, 127] => {
						status.button(*button, true);
					},
					[128, button, 64] => {
						status.button(*button, false);
					},
					[176, analog, value] => {
						status.analog(*analog, *value);
					},
					_ => println!("can't understand message"),
				}
				println!("{}: {:?} (len = {})", stamp, message, message.len());
			}, ()).unwrap();

			let status = Arc::clone(&status_arc);
			loop {
				let mut request_pipe = NamedPipe::new(r"\\.\pipe\ipc-request").unwrap();
				request_pipe.connect().unwrap();

				let mut buffer: [u8; 1] = [255; 1];
				request_pipe.read_exact(&mut buffer).unwrap();

				let status = status.read().unwrap();
				request_pipe.write_all(&status as &[u8; size_of::<Status>()]).unwrap();
			}
		}
		if argument == "client" {
			let mut request_pipe = connect(r"\\.\pipe\ipc-request").unwrap();

			request_pipe.write_all(&[1]).unwrap();

			let mut status = Status::default();
			request_pipe.read_exact(&mut status as &mut [u8; size_of::<Status>()]).unwrap();

			println!("{:?}", status);
		}
	}
}
