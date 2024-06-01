use std::{env, error::Error, path::PathBuf, process::exit};
use std::fs::File;
use std::io::{BufWriter, Write};
use image::GenericImageView;

fn main() -> Result<(), Box<dyn Error>> {
    let mut image_path: Option<String> = None;
    let mut grid_size: (u32, u32) = (12, 12);
    let mut title: String = String::from("default");

    for arg in env::args() {
        dbg!(&arg);
        if arg.starts_with("--image=") {
            image_path = Some(arg.clone().split_off(8))
        }
        if arg.starts_with("--grid-size=") {
            let grid_val = arg.clone().split_off(12);
            let mut grid_val = grid_val.split('x');
            grid_size =
                (grid_val.next().unwrap().parse()?,
                 grid_val.next().unwrap().parse()?);
        }
        if arg.starts_with("--name=") {
            title = arg.clone().split_off(7);
        }
    }

    dbg!(&image_path, &grid_size, &title);

    if let None = image_path {
        eprintln!("Please provide a file!");
        exit(1);
    }

    let image_path = PathBuf::from(image_path.unwrap()).canonicalize()?;
    dbg!(&image_path);

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
    let file = File::create(format!("{title}.svg")).expect("Check index.html if it exists.");
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
