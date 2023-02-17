use std::env;
use std::fs;
use std::process::exit;
use std::error::Error;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use tokio::{self, net::{TcpListener, TcpStream}, io::{BufReader, AsyncBufReadExt, AsyncWriteExt}};
use rand::Rng;
use serde::Serialize;
use once_cell::sync::OnceCell;

static PATH: OnceCell<String> = OnceCell::new();

struct Config {
    pub path: String,
    pub port: u16,
}

#[derive(Serialize)]
struct Matrix(Vec<Vec<f64>>);

impl Matrix {
    pub fn new(string: &str) -> Self {
        let rows: Vec<_> = string.split("\n").collect();

        let mut res = Self(vec![vec![0.0; rows.len()]; rows.len()]);

        for (i, row) in rows.iter().enumerate() {
            let columns: Vec<_> = row.split(", ").collect();

            for (j, val) in columns.iter().enumerate() {
                res.0[i][j] = match val.parse() {
                    Ok(val) => val,
                    Err(_) => 0.0,
                };
            }
        }

        res
    }

    pub fn zero(n: usize) -> Self {
        Self(vec![vec![0.0; n]; n])
    }

    pub fn lu(&self) -> (Self, Self) {
        let n = self.0.len();
        let mut l = Self::zero(n);
        let mut u = Self::zero(n);

        for i in 0..n {
            for j in 0..n {
                if j < i {
                    l.0[j][i] = 0.0;
                } else {
                    u.0[i][j] = self.0[i][j];
                    for k in 0..i {
                        u.0[i][j] -= l.0[i][k] * u.0[k][j];
                    }
                }
            }
            for j in i..n {
                if j < i {
                    l.0[j][i] = 0.0;
                } else {
                    l.0[j][i] = self.0[j][i];
                    for k in 0..i {
                        l.0[j][i] -= l.0[j][k] * u.0[k][i];
                    }
                    l.0[j][i] /= u.0[i][i];
                }
            }
        }

        (l, u)
    }
}

fn parse_args(args: Vec<String>) -> Result<Config, Box<dyn Error>> {
    if args.len() != 3 {
        return Err("")?;
    }

    Ok(Config {
        path: args[1].clone(),
        port: args[2].parse()?,
    })
}

async fn start_server(config: Config) -> Result<(), Box<dyn Error>> {
    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), config.port);
    let listener = TcpListener::bind(socket).await?;
    PATH.set(config.path)?;

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let _ = process_request(stream).await;
        });
    }
}

async fn process_request(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = String::new();
    BufReader::new(&mut stream).read_line(&mut buf).await?;

    let response = tokio::task::spawn_blocking(move || {
        generate_response()
    }).await?;

    let response = match response {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{}", e);
            String::from("HTTP/1.1 500 Internal Server Error\r\n\r\n")
        },
    };

    stream.write(response.as_bytes()).await?;
    stream.flush().await?;

    Ok(())
}

fn generate_response() -> Result<String, Box<dyn Error + Send + Sync>> {
    let index = {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..(fs::read_dir(PATH.get().unwrap())?.count()))
    };

    let matrix = String::from_utf8(fs::read(format!("{}/{}.dat", PATH.get().unwrap(), index))?)?;


    let matrix = Matrix::new(&matrix);

    let (l, u) = matrix.lu();
    let l = serde_json::to_string(&l)?;
    let u = serde_json::to_string(&u)?;

    Ok(format!("HTTP/1.1 200 OK\r\n\r\nA={}\r\n\r\nB={}\r\n\r\n", l, u))
}

#[tokio::main]
async fn main() {
    let args = env::args().collect();

    let config = match parse_args(args) {
        Ok(config) => config,
        Err(_) => {
            eprintln!("Usage: server PATH PORT");
            exit(1)
        },
    };

    match start_server(config).await {
        Ok(()) => return,
        Err(_) => {
            eprintln!("Server failed to start or crashed. ");
            exit(1);
        },
    };
}
