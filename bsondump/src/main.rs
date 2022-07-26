use std::{
    error::Error,
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, BufWriter, Write},
    result::Result,
};

use clap::{ArgEnum, Parser};
use clap_verbosity_flag::Verbosity;
use log::{error, info};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
#[clap(rename_all = "camelCase")]
enum OutputType {
    Debug,
    Json,
    PrettyJson,
}

#[derive(Parser)]
#[clap(rename_all = "camelCase")]
struct Cli {
    /// Path to BSON file to dump to JSON; default is stdin
    file: Option<String>,

    #[clap(flatten)]
    verbose: Verbosity,

    #[clap(name="type", long="type", arg_enum, default_value_t = OutputType::Json)]
    // type of output: debug, json, prettyJson
    output_type: OutputType,

    #[clap(long)]
    /// Validate BSON during processing
    objcheck: bool,

    #[clap(long = "outFile", name = "outFile")]
    /// Path to output file to dump JSON to; default is stdout
    out_file: Option<String>,
}

fn print_error_and_exit(num_found: u32, message: String) {
    info!("{} objects found", num_found);
    error!("{}", message);
    std::process::exit(1);
}

fn print_json<W: Write>(
    writer: &mut W,
    raw_doc_buf: &bson::RawDocumentBuf,
    num_found: u32,
    pretty: bool,
    exit_on_error: bool,
) {
    let result = bsondump::to_canonical_extjson_value(raw_doc_buf);
    if let Err(err) = result {
        if exit_on_error {
            print_error_and_exit(num_found, format!("Failed to convert to canonical extended json: {}", err));
        }
        return;
    }
    let value = result.unwrap();

    if !pretty {
        if let Err(err) = writeln!(writer, "{}", value) {
            print_error_and_exit(num_found, format!("{}", err));
        }
        return;
    }

    let result = bsondump::to_pretty_string(&value);
    if let Err(err) = result {
        if exit_on_error {
            print_error_and_exit(num_found, format!("{}", err));
        }
        return;
    }
    let value = result.unwrap(); // no error here

    if let Err(err) = writeln!(writer, "{}", value) {
        print_error_and_exit(num_found, format!("{}", err));
    }

    if let Err(err) = writer.flush() {
        print_error_and_exit(num_found, format!("{}", err));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    env_logger::Builder::new().filter_level(cli.verbose.log_level_filter()).init();

    let mut reader: Box<dyn BufRead> = match cli.file.as_deref() {
        None => Box::new(BufReader::new(stdin())),
        Some(path) => match File::open(path) {
            Err(err) => {
                error!("Failed to open {path} for reading. {err}", path = path, err = err);
                std::process::exit(1);
            }
            Ok(file) => Box::new(BufReader::new(file)),
        },
    };

    let mut writer: Box<dyn Write> = match cli.out_file.as_deref() {
        None => Box::new(BufWriter::new(stdout())),
        Some(path) => match File::create(path) {
            Err(err) => {
                error!("Failed to create {path} for writing. {err}", path = path, err = err);
                std::process::exit(1);
            }
            Ok(file) => Box::new(BufWriter::new(file)),
        },
    };

    let mut num_found = 0;
    for result in bsondump::docbytes::source(&mut reader) {
        if let Err(ref err) = result {
            print_error_and_exit(num_found, format!("{}", err));
        }
        let bson_bytes = result.unwrap();  // No error here

        let result = bson::RawDocumentBuf::from_bytes(bson_bytes.bytes);
        if let Err(ref err) = result {
            print_error_and_exit(num_found, format!("{}", err));
        }
        let raw_doc_buf = result.unwrap(); // No error here

        match cli.output_type {
            OutputType::Json => {
                print_json(&mut writer, &raw_doc_buf, num_found, false, cli.objcheck);
            }
            OutputType::PrettyJson => {
                print_json(&mut writer, &raw_doc_buf, num_found, true, cli.objcheck);
            }
            OutputType::Debug => {
                let result = bsondump::debug(&raw_doc_buf);
                if let Err(ref err) = result {
                    print_error_and_exit(num_found, format!("{}", err));
                }
                let value = result.unwrap();
                if let Err(err) = writeln!(writer, "{}", value) {
                    print_error_and_exit(num_found, format!("{}", err));
                }
                if let Err(err) = writer.flush() {
                    print_error_and_exit(num_found, format!("{}", err));
                }
            }
        };

        num_found += 1;
    }
    info!("{} objects found", num_found);

    Ok(())
}
