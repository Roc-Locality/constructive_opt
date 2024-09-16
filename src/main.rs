#![allow(dead_code)]
use std::error::Error;
use std::fs::File;
use std::process::Command;
use csv::ReaderBuilder;
use indicatif::ProgressBar;
use rand::Rng;
use serde::Deserialize;
use constructive_opt::opt_miss_ratio;

#[derive(Debug, Deserialize)]
struct RawAccessTrace {
    address: String,
}

fn read_third_column_as_usize_vec(file_path: &str) -> Result<Vec<usize>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    let mut trace = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let value: usize = usize::from_str_radix(&record[2], 16)?;
        trace.push(value);
    }

    Ok(trace)
}

fn generate_opt_miss_ratio_data(
    trace: &[usize],
    max_cache_size: usize,
    output_csv: &str,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(output_csv)?;

    wtr.write_record(&["cache_size", "miss_ratio"])?;

    let bar = create_progress_bar(max_cache_size);
    for cache_size in 1..=max_cache_size {
        bar.inc(1);
        let miss_ratio = opt_miss_ratio(&trace, cache_size);
        wtr.write_record(&[cache_size.to_string(), miss_ratio.to_string()])?;
    }
    bar.finish();

    wtr.flush()?;
    Ok(())
}

// fn draw_opt_miss_ratio_curve(
//     trace: &[usize],
//     max_cache_size: usize,
//     output_path: &str,
// ) -> Result<(), Box<dyn Error>> {
//     let root = BitMapBackend::new(output_path, (1280, 960)).into_drawing_area();
//     root.fill(&WHITE)?;
//
//     let mut chart = ChartBuilder::on(&root)
//         .caption("OPT Miss Ratio Curve", ("sans-serif", 50).into_font())
//         .margin(5)
//         .x_label_area_size(30)
//         .y_label_area_size(30)
//         // .build_cartesian_2d(0f64..max_cache_size as f64, 0f64..1f64)?;
//         .build_cartesian_2d(
//             (1f64..(max_cache_size as f64).log2()).log_scale(),
//             0.01f64..1f64,
//         )?;
//
//     // chart.configure_mesh().draw()?;
//     chart
//         .configure_mesh()
//         .x_desc("Cache Size")
//         .x_label_formatter(&|&x| format!("{}", (2f64.powi(x as i32)) as usize)) // Format labels as 2^n
//         .draw()?;
//
//     let bar = create_progress_bar(max_cache_size);
//     chart
//         .draw_series(LineSeries::new(
//             (1..=max_cache_size).map(|x| {
//                 bar.inc(1);
//                 // (x as f64, opt_miss_ratio(&trace, x))
//                 ((x as f64).log2(), opt_miss_ratio(&trace, x)) // Use log2 for x values
//             }),
//             &RED,
//         ))?
//         .label("OPT Miss Ratio Curve")
//         .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
//     bar.finish();
//     chart
//         .configure_series_labels()
//         .background_style(&WHITE.mix(0.8))
//         .border_style(&BLACK)
//         .draw()?;
//
//     root.present()?;
//     Ok(())
// }

fn create_progress_bar(max_cache_size: usize) -> ProgressBar {
    let bar = ProgressBar::new(max_cache_size as u64);
    bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:60.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );
    bar
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let _trace = read_third_column_as_usize_vec("./out/access_trace.csv")?;

    let _trace: Vec<usize> = (0..1024)
        .map(|_| rand::thread_rng().gen_range(0..128))
        .collect();

    // let _trace = vec![1, 2, 3, 4, 5, 1, 2, 3, 4, 1, 2, 3, 4];

    // draw_opt_miss_ratio_curve(&_trace, 256, "out/opt-miss-ratio-curve-plot.png")?;


    let max_cache_size = 256;
    let data_csv = "out/access_trace_miss.csv";
    let output_plot = "out/opt_miss_ratio_curve_plot.png";

    // Generate data and save to CSV
    generate_opt_miss_ratio_data(&_trace, max_cache_size, data_csv)?;

    println!("Data generated and saved to {}", data_csv);

    // Call the Python script to generate the plot
    Command::new("venv/bin/python")
        .arg("src/plot_opt_miss_ratio.py")
        .arg(data_csv)
        .arg(output_plot)
        .status()?;

    Ok(())
}
