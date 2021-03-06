use image::ImageBuffer;
use std::process::exit;
use std::io::SeekFrom;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use image::Rgba;
use std::env;
use std::f64;

const LEN:u32=600;
const SPR_RAD:u32=10;
const MID:u32=LEN/2;
const PADDING:u32=8;
const LIMIT:u64=25;
const AMP:f64=10.0;

struct Sprinkle{
	color:Rgba<u8>,
	x:u32,y:u32,
	r:f64
}
struct Donut{
	sprinkles:Vec<Sprinkle>,
	frosting:Rgba<u8>,
	border:Rgba<u8>,
	darker:Rgba<u8>,
	color:Rgba<u8>
}

mod flavors{
	use image::Rgba;
	pub struct Flavor{ pub main:Rgba<u8>,pub dark:Rgba<u8> }
	pub const VANILLA:Flavor=Flavor{
		dark:Rgba{data:[235,235,235,255]},
		main:Rgba{data:[255,255,255,255]}
	};
	pub const CHOCOLATE:Flavor=Flavor{
		main:Rgba{data:[125,50,17,255]},
		dark:Rgba{data:[94,37,12,255]}
	};
	pub const DOUGH:Flavor=Flavor{
		dark:Rgba{data:[189,184,100,255]},
		main:Rgba{data:[245,240,142,255]}
	};
	pub const PINK:Flavor=Flavor{
		dark:Rgba{data:[196,106,136,255]},
		main:Rgba{data:[237,142,174,255]}
	};
}

// Helper functions
fn dist(x1:u32,y1:u32,x2:u32,y2:u32) -> u32 {
		return (((x1 as i64-x2 as i64).pow(2)+(y1 as i64-y2 as i64).pow(2)) as f64).sqrt() as u32
}
fn frosted_ring(x:u32,y:u32) -> u32 {
	let dy=(MID as f64)-(y as f64);
	let dx=(MID as f64)-(x as f64);
	let a=dy.atan2(dx)*(f64::consts::PI*4.0);
	return ((a.cos()*AMP)+(MID as f64/2.0)) as u32
}

// Main algorithm
fn build_donut(mut f:File,l:u64) -> Donut {
	let mut donut:Donut=Donut{
		frosting:flavors::CHOCOLATE.main,
		darker:flavors::CHOCOLATE.dark,
		border:flavors::DOUGH.dark,
		color:flavors::DOUGH.main,
		sprinkles:Vec::new()
	};
	
	// Set dough and frosting colors
	let n=(l as f64).sqrt() as u64;
	if n<LIMIT {
		donut.border=flavors::CHOCOLATE.dark;
		donut.color=flavors::CHOCOLATE.main
	}
	if n%2==0 {
		donut.frosting=flavors::VANILLA.main;
		donut.darker=flavors::VANILLA.dark
	}else if n%3==0 {
		donut.frosting=flavors::PINK.main;
		donut.darker=flavors::PINK.dark
	}
	
	// Add sprinkles
	let mut buffer=[0;25];
	let m=MID as f64;
	for i in 0..LIMIT{
		let di=if n<LIMIT { 1 } else { n/LIMIT };
		let _=f.seek(SeekFrom::Start(((i*di*n) as u64)%l)).unwrap();
		f.read(&mut buffer).unwrap();
		if n<LIMIT {
			for j in n..LIMIT {
				buffer[j as usize]=buffer[(j-n) as usize]
			}
		}
		let a=(buffer[0] as f64)*(buffer[1] as f64)*(buffer[2] as f64)*(buffer[3] as f64);
		let mut r=(buffer[4] as f64)*(buffer[5] as f64)*(buffer[6] as f64)*(buffer[7] as f64);
		let rot=(buffer[8] as f64)*(buffer[9] as f64)*(buffer[10] as f64)*(buffer[11] as f64);
		let red=((buffer[12] as u64)*(buffer[13] as u64))%255;
		let green=((buffer[14] as u64)*(buffer[15] as u64))%255;
		let blue=((buffer[16] as u64)*(buffer[17] as u64))%255;
		r%=(m/2.0)-(SPR_RAD as f64*2.0)-(PADDING as f64*2.0)-AMP;
		r+=(m/2.0)+(SPR_RAD as f64)+(PADDING as f64)+AMP;
		donut.sprinkles.push(Sprinkle{
			color:Rgba{data:[red as u8,green as u8,blue as u8,255]},
			x:((r*a.cos())+m) as u32,
			y:((r*a.sin())+m) as u32,
			r:rot
		})
	}
	return donut
}
fn render_donut(donut:Donut,path:&String) -> () {
	let mut img:ImageBuffer<Rgba<u8>,Vec<u8>>=ImageBuffer::new(LEN,LEN);
	let blank:Rgba<u8>=Rgba{data:[255,255,255,255]};
	for x in 0..LEN {
		for y in 0..LEN {
			let d=dist(x,y,MID,MID);
			let mut color:Rgba<u8>=donut.color;
			if d>MID || d<MID/3 { color=blank }
			else if d>MID-PADDING { color=donut.darker }
			else if d<(MID/3)+PADDING { color=donut.border }
			else{
				let ring=frosted_ring(x,y);
				if d>ring {				
					color=donut.frosting;
					if d<ring+PADDING {
						color=donut.darker					
					}
				}
			}
			img.put_pixel(x,y,color)
		}
	}
	
	// Render sprinkles
	for s in &donut.sprinkles {
		let dx=(SPR_RAD as f64*s.r.cos()) as i32;
		let dy=(SPR_RAD as f64*s.r.sin()) as i32;
		for x in s.x-SPR_RAD..s.x+SPR_RAD{
			for y in s.y-SPR_RAD..s.y+SPR_RAD{
				let d=dist(x,y,s.x,s.y);
				let a=((y as f64)-(s.y as f64)).atan2((x as f64)-(s.x as f64));
				if d<SPR_RAD {
					img.put_pixel((x as i32+dx) as u32,(y as i32+dy) as u32,s.color);
					img.put_pixel((x as i32-dx) as u32,(y as i32-dy) as u32,s.color)
				}
				let x1=((d as f64*(a+s.r).cos()) as i32+(s.x as i32)) as u32;
				let y1=((d as f64*(a+s.r).sin()) as i32+(s.y as i32)) as u32;
				img.put_pixel(x1+1,y1  ,s.color);
				img.put_pixel(x1-1,y1  ,s.color);
				img.put_pixel(x1  ,y1+1,s.color);
				img.put_pixel(x1  ,y1-1,s.color);
				img.put_pixel(x1  ,y1  ,s.color);
				img.put_pixel(x1+1,y1+1,s.color);
				img.put_pixel(x1+1,y1-1,s.color);
				img.put_pixel(x1-1,y1+1,s.color);
				img.put_pixel(x1-1,y1-1,s.color)
			}
		}
	}
	let _=img.save(path).unwrap();
	()
}

// Main
fn main() {
	let args:Vec<String>=env::args().collect();
	if args.len()<2 || args.len()>3 {
		println!("Usage: donut input [output]");
		exit(1)
	}
	let mut input=File::open(&args[1]).unwrap();
	let len=input.seek(SeekFrom::End(0)).unwrap();
	if len>1000000 {
		let mb=len/1000000;
		println!("File passed of length {}MB",mb)
	}else{
		println!("File passed of length {} bytes",len)
	}
	let _=input.seek(SeekFrom::Start(0));
	println!("Encrypting file...");
	let donut=build_donut(input,len);
	let mut output:&String=&String::from("../donut.png");
	if args.len()>2 { output=&args[2] }
	println!("Rendering...");
	render_donut(donut,output);
	println!("Algorithm ran to completion");
	println!("Donut written to {}",output)
}
