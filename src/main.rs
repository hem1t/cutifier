use std::{error::Error, path::PathBuf, process::exit};
use std::fs::File;
use std::io::{BufWriter, Write};
use clap::Parser;
use image::GenericImageView;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    /// The input image, that you want to rasterise.
    #[arg(short, long, value_name="FILE")]
    image: PathBuf,

    /// Output Pixel size
    #[arg(short, long)]
    grid_size: Option<String>,

    /// Name of output file.
        #[arg(short, long)]
    name: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Cli::parse();

    let image_path = options.image.canonicalize()?;

    let mut grid_size: (u32, u32) = (12, 12);
    if let Some(gs) = options.grid_size {
        let mut gs = gs.split('x');
        grid_size.0 = gs.next().unwrap().parse::<u32>()?;
        grid_size.1 = gs.next().unwrap().parse::<u32>()?;
    }

    let mut name = String::from("default");
    if let Some(n) = options.name {
        name = n;
    }

    dbg!(&image_path, &grid_size, &name);

    // check image type
    match imghdr::from_file(image_path.clone()) {
        Ok(Some(imghdr::Type::Png)) => (),
        Ok(Some(imghdr::Type::Jpeg)) => (),
        // Ok(Some(imghdr::Type::Gif)) => (),
        _ => {
            eprintln!("Only supporting types 'Png' and 'Gif', right now!");
            exit(1);
        }
    }

    // OPEN image
    let img = image::io::Reader::open(image_path)?.decode()?;

    // reduce grid size
    // step 1: get width and height
    let (width, height) = dbg!((img.width(), img.height()));
    // step 2: calculate available grids
    let (w_g, h_g) = ((width as f64 / grid_size.0 as f64).ceil() as u32, (height as f64 / grid_size.1 as f64).ceil() as u32);
    // step 3: iterate over grids
    let (mut g_x, mut g_y) = (0, 0);
    let mut matrix: Vec<Vec<RGBA>> = vec![Vec::with_capacity(w_g as usize); h_g as usize];
    // when out of height then break
    while g_y < height {
        // get averaged out pixel of the grid and push to the Matrix.
        let mut vrgb = Vec::with_capacity((grid_size.0 * grid_size.1) as usize);
        for x in g_x..(g_x + grid_size.0) {
            for y in g_y..(g_y + grid_size.1) {
                if x < width && y < height {
                    let p = img.get_pixel(x, y);
                    vrgb.push(RGBA(p.0[0], p.0[1], p.0[2], p.0[3]));
                }
            }
        }
        matrix[(g_y / grid_size.1) as usize].push(RGBA::from_vrgb(vrgb));
        // goto next grid
        g_x += grid_size.0;
        if g_x >= width {
            g_x = 0;
            g_y += grid_size.1;
        }
    }

    // println!("{:?}, \nmx: {}, my: {}", matrix, matrix.len(), matrix[0].len());

    // Create SVG circles of same size.
    let file = File::create(format!("{name}.svg")).expect("Check index.html if it exists.");
    let mut file_buffer = BufWriter::new(file);

    file_buffer.write(format!("<svg height=\"{height}\" width=\"{width}\" xmlns=\"http://www.w3.org/2000/svg\">").as_bytes()).expect("file init write failed!");

    let r = grid_size.0 as f64 / 2.0;
    for y in 0..(h_g as usize) {
        for x in 0..(matrix[y].len()) {
            let color = matrix[y][x].to_str();
            let (cx, cy) = (x as f64 * (2.0*r) + r, y as f64 * (2.0*r) + r);
            // dbg!((cx, cy, x, y));
            file_buffer.write(format!("<circle r=\"{r}\" cx=\"{cx}\" cy=\"{cy}\" fill=\"{color}\" />").as_bytes()).expect(format!("Failed write at cx:{}, cy:{}", cx, cy).as_str());
        }
    }

    file_buffer.write(format!("</svg>").as_bytes()).expect("End write failed!");
    file_buffer.flush().expect("flushing failed");
    Ok(())
}


#[derive(Clone, Debug)]
struct RGBA (u8,u8,u8, u8);

impl RGBA {
    fn from_vrgb(vrgb: Vec<RGBA>) -> Self {
        let r: u64 = vrgb.iter().map(|p| p.0 as u64).sum::<u64>() / vrgb.len() as u64;
        let g: u64 = vrgb.iter().map(|p| p.1 as u64).sum::<u64>() / vrgb.len() as u64;
        let b: u64 = vrgb.iter().map(|p| p.2 as u64).sum::<u64>() / vrgb.len() as u64;
        let a: u64 = vrgb.iter().map(|p| p.3 as u64).sum::<u64>() / vrgb.len() as u64;
        RGBA(r as u8, g as u8, b as u8, a as u8)
    }

    fn to_str(&self) -> String {
        format!("rgba({}, {}, {}, {})", self.0, self.1, self.2, self.3)
    }
}
