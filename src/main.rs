//! Toy C compiler targets ARM on Linux and macOS
//!
//! Use system assembler and linker to assemble and link the executable file

#[macro_use]
extern crate log;

use std::fs;

use clap::{App, Arg};
use env_logger::{Builder, Env};
use parse::parse;
use scan::scan;

mod ast;
mod common;
mod parse;
mod scan;

fn main() {
    let opts = parse_opts();
    init_logger(&opts);

    // validate
    ensure_input_exist(&opts.files);

    // always execute front-end to emit asm
    exec_cc1(&opts);

    // stop if -S
    if opts.compile_only {
        return;
    }

    // run assembler
    run_assembler(&opts);

    // stop if -c
    if opts.complie_as_only {
        return;
    }

    // run linker
    run_linker(&opts);
}

/// compiler front-end to emit assembly code
/// phases: scanning -> parsing -> semantics analysis -> code generation (ARM ASM)
fn exec_cc1(opts: &Opts) {
    info!("execute core cc1");
    opts.files.iter().for_each(|f| {
        let contents = fs::read_to_string(f).unwrap();
        let toks = scan(&contents);
        let ast = parse(toks);
        debug!("{:#?}", ast)
    })
}

/// use system assembler (GNU as) to assemble asm code to object code
fn run_assembler(opts: &Opts) {
    info!("invoke assembler")
    // TODO implement
}

/// use system linker (GNU ld) to link object code to machine code (ELF)
fn run_linker(opts: &Opts) {
    info!("invoke linker")
    // TODO implement
}

fn ensure_input_exist(files: &[String]) {
    files.iter().for_each(|f| {
        let _ = fs::metadata(f).expect(format!("invalid input file: {}", f).as_str());
    });
}

#[derive(Debug, Default)]
struct Opts {
    files: Vec<String>,
    output: String,
    compile_only: bool,
    complie_as_only: bool,
    debug: bool,
    verbose: bool,
}

fn parse_opts() -> Opts {
    let app = create_arg_app();
    let args = app.get_matches();

    // load options
    Opts {
        compile_only: args.is_present("arg-S"),
        complie_as_only: args.is_present("arg-c"),
        debug: args.is_present("debug"),
        verbose: args.is_present("verbose"),
        output: args
            .value_of("output")
            .unwrap_or_else(|| "a.out")
            .to_string(),
        files: args
            .values_of("input")
            .unwrap()
            .map(|v| v.to_string())
            .collect(),
    }
}

fn create_arg_app() -> App<'static> {
    App::new("tncc")
        .author("Tri Nguyen")
        .author("Toy C compiler in Rust targets ARM (Linux and macOS)")
        .arg(
            Arg::new("arg-S")
                .short('S')
                .about("Emit assembly only; do not run assembler or linker"),
        )
        .arg(
            Arg::new("arg-c")
                .short('c')
                .about("Emit assembly and run assembler; do not run linker"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .value_name("file")
                .about("Output path"),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .about("print verbose logging"),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .about("print debug logging"),
        )
        .arg(
            Arg::new("input")
                .required(true)
                .multiple(true)
                .about("input C source files"),
        )
}

fn init_logger(opts: &Opts) {
    use std::io::Write;
    let level = if opts.debug {
        "debug"
    } else if opts.verbose {
        "info"
    } else {
        "warn" // default
    };
    let env = Env::default().filter_or("MY_LOG_LEVEL", level);
    Builder::from_env(env)
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();
}
