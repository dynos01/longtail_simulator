use std::env;
use std::fs;
use std::process::exit;
use std::error::Error;
use rand::Rng;

struct Config {
    pub path: String,
    pub count: usize,
    pub min_size: u16,
    pub max_size: u16,
}

fn parse_args(args: Vec<String>) -> Result<Config, Box<dyn Error>> {
    if args.len() != 5 {
        return Err("")?;
    }

    Ok(Config {
        path: args[1].clone(),
        count: args[2].parse()?,
        min_size: args[3].parse()?,
        max_size: args[4].parse()?,
    })
}

fn generate_matrix(config: Config) -> Result<(), Box<dyn Error>> {
    let mut rng = rand::thread_rng();

    for i in 0..config.count {
        let size = rng.gen_range(config.min_size..(config.max_size + 1)) + 1;

        let mut matrix = String::from("");

        for _j in 0..size {
            let mut line = String::from("");
            line.push_str(&(rng.gen::<f64>().to_string()));

            for _k in 1..size {
                line.push_str(&format!(", {}", rng.gen::<f64>().to_string()));
            }

            matrix.push_str(&format!("{}\n", line));
        }

        fs::write(format!("{}/{}.dat", config.path, i), matrix)?;
    }

    Ok(())
}

fn main() {
    let args = env::args().collect();

    let config = match parse_args(args) {
        Ok(config) => config,
        Err(_) => {
            println!("Usage: matrix_generator PATH COUNT MINIMUM_SIZE MAXIMUM_SIZE");
            exit(1)
        },
    };

    match generate_matrix(config) {
        Ok(()) => return,
        Err(_) => {
            println!("Failed to generate matrices. ");
            exit(1);
        },
    };
}
