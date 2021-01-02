//! Toy C compiler targets ARM on Linux and macOS
//!
//! Use system assembler and linker to assemble and link the executable file

#[macro_use]
extern crate log;

use std::{fs, path::PathBuf};
use std::{fs::File, io::prelude::*};

use clap::{App, Arg};
use codegen::gen_asm;
use env_logger::{Builder, Env};
use parse::parse;
use scan::scan;
use semantics::analyse;
use util::*;

mod ast;
mod codegen;
mod common;
mod parse;
mod scan;
mod semantics;
mod util;

fn main() {
    let opts = parse_opts();
    init_logger(&opts);

    // validate
    ensure_input_exist(&opts.files);

    // always execute front-end to emit asm
    let target = TargetOs::current();
    exec_cc1(&opts, &target);

    // stop if -S
    if opts.compile_only {
        return;
    }

    // check arch
    check_target(&target);

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
fn exec_cc1(opts: &Opts, target: &TargetOs) {
    info!("execute core cc1");
    for f in &opts.files {
        let contents = fs::read_to_string(f).unwrap();

        // scan to tokens
        debug!("start scanning...");
        let toks = scan(&contents);

        // parse to ast
        debug!("start parsing...");
        let mut ast = parse(toks);
        debug!("{:#?}", ast);

        // semantics analysis and type checking
        debug!("start semantics analysis");
        analyse(&mut ast);

        // generate asm
        debug!("start code generation...");
        let asm = gen_asm(&ast, target);
        debug!("{}", asm);

        // write to output
        write_asm_file(&asm, &opts.output, f);
        info!("finished generating ARM assembly");
    }
}

fn write_asm_file(asm: &String, output: &Option<PathBuf>, f: &PathBuf) {
    let mut fout = match &output {
        Some(p) => File::create(p),
        _ => File::create(new_output_asm(f)),
    }
    .unwrap();

    fout.write_all(asm.as_bytes()).expect("wrote asm to file");
}

/// use system assembler (GNU as) to assemble asm code to object code
fn run_assembler(opts: &Opts) {
    info!("invoke assembler")
}

/// use system linker (GNU ld) to link object code to machine code (ELF)
fn run_linker(opts: &Opts) {
    info!("invoke linker")
    // TODO implement
}

/// support macos arm and linux arm only
fn check_target(target: &TargetOs) {
    match (target, util::is_aarch64()) {
        (TargetOs::MacOs, true) => (),
        (TargetOs::Linux, true) => (),
        (os, _) => panic!("Current OS ({:?}) and arch is not yet supported, try macos or linux or aarch64 instead", os)
    }
}

fn ensure_input_exist(files: &[PathBuf]) {
    files.iter().for_each(|f| {
        if !f.is_file() {
            panic!("invalid input file '{}'", f.to_str().unwrap())
        }
    });
}

#[derive(Debug, Default)]
struct Opts {
    files: Vec<PathBuf>,
    output: Option<PathBuf>,
    compile_only: bool,
    complie_as_only: bool,
    debug: bool,
    verbose: bool,
}

fn parse_opts() -> Opts {
    let app = create_arg_app();
    let args = app.get_matches();

    // load options
    let out = match args.value_of("output") {
        Some(v) => Some(PathBuf::from(v.to_string())),
        _ => None,
    };
    Opts {
        compile_only: args.is_present("arg-S"),
        complie_as_only: args.is_present("arg-c"),
        debug: args.is_present("debug"),
        verbose: args.is_present("verbose"),
        output: out,
        files: args
            .values_of("input")
            .unwrap()
            .map(|v| PathBuf::from(v))
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
