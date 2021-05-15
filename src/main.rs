use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Instant;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "tpctools", about = "Tools for generating TPC data sets.")]
struct Opt {
    /// Scale factor
    #[structopt(short, long)]
    scale: usize,

    /// Number of partitions to generate in parallel
    #[structopt(short, long)]
    partitions: usize,

    /// Path to tpcds-kit
    #[structopt(short, long, parse(from_os_str))]
    tpcdskit_path: PathBuf,

    /// Output path
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    let scale = opt.scale;
    let partitions = opt.partitions;
    let kitpath = opt.tpcdskit_path.as_path().to_str().unwrap().to_string();
    let output_path_str = opt.output.as_path().to_str().unwrap().to_string();

    let gen = TpcDs::new(scale, partitions, kitpath, output_path_str);
    gen.generate()?;

    Ok(())
}

trait Tpc {
    fn generate(&self) -> Result<()>;
}

struct TpcDs {
    scale: usize,
    partitions: usize,
    tpcdskit_path: String,
    output_path_str: String,
}

impl TpcDs {
    fn new(
        scale: usize,
        partitions: usize,
        tpcdskit_path: String,
        output_path_str: String,
    ) -> Self {
        Self {
            scale,
            partitions,
            tpcdskit_path,
            output_path_str,
        }
    }
}

impl Tpc for TpcDs {
    fn generate(&self) -> Result<()> {
        let mut handles = vec![];

        let start = Instant::now();

        for i in 1..=self.partitions {
            let tpcdskit_path = self.tpcdskit_path.clone();
            let output_path_str = self.output_path_str.clone();
            let scale = self.scale;
            let partitions = self.partitions;
            handles.push(thread::spawn(move || {
                let output = Command::new("./dsdgen")
                    .current_dir(tpcdskit_path)
                    .arg("-FORCE")
                    .arg("-DIR")
                    .arg(output_path_str)
                    .arg("-SCALE")
                    .arg(format!("{}", scale))
                    .arg("-CHILD")
                    .arg(format!("{}", i))
                    .arg("-PARALLEL")
                    .arg(format!("{}", partitions))
                    .output()
                    .expect("failed to generate data");

                println!("{:?}", output);
            }));
        }

        // wait for all threads to finish
        for h in handles {
            h.join().unwrap();
        }

        let duration = start.elapsed();

        println!(
            "Generated TPC-DS data at scale factor {} with {} partitions in: {:?}",
            self.scale, self.partitions, duration
        );

        let tables = [
            "call_center",
            "catalog_page",
            "catalog_sales",
            "catalog_returns",
            "customer",
            "customer_address",
            "customer_demographics",
            "date_dim",
            "income_band",
            "household_demographics",
            "inventory",
            "store",
            "ship_mode",
            "reason",
            "promotion",
            "item",
            "store_sales",
            "store_returns",
            "web_page",
            "warehouse",
            "time_dim",
            "web_site",
            "web_sales",
            "web_returns",
        ];

        for table in &tables {
            let output_dir = format!("{}/{}", self.output_path_str, table);
            if !Path::new(&output_dir).exists() {
                println!("Creating directory {}", output_dir);
                fs::create_dir(&output_dir)?;
            }
            for i in 1..=self.partitions {
                let filename = format!(
                    "{}/{}_{}_{}.dat",
                    self.output_path_str, table, i, self.partitions
                );
                let filename2 = format!("{}/part-{}.dat", output_dir, i);
                if Path::new(&filename).exists() {
                    println!("mv {} {}", filename, filename2);
                    fs::rename(filename, filename2)?;
                }
            }
        }

        Ok(())
    }
}

// struct TpcH {
// git@github.com:databricks/tpch-dbgen.git
// }