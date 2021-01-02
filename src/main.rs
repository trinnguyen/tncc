//! Toy C compiler targets ARM on Linux and macOS
//!
//! Use system assembler and linker to assemble and link the executable file

#[macro_use]
extern crate log;

use std::{
    fs,
    path::PathBuf,
    process::Command,
};
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
    let asm_paths = exec_cc1(&opts);

    // stop if -S
    if opts.compile_only {
        return;
    }

    // check arch
    check_target(&target);

    // run assembler
    let obj_paths = run_assembler(&opts, &asm_paths);

    // stop if -c
    if opts.complie_as_only {
        return;
    }

    // run linker
    let out = run_linker(&opts, &obj_paths);
    info!("ouput at {:?}", out);
}

/// compiler front-end to emit assembly code
/// phases: scanning -> parsing -> semantics analysis -> code generation (ARM ASM)
fn exec_cc1(opts: &Opts) -> Vec<PathBuf> {
    info!("execute core cc1");
    opts.files
        .iter()
        .map(|f| {
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
            let asm = gen_asm(&ast, &opts.target);
            debug!("{}", asm);

            // write to output
            write_asm_file(&asm, opts, f)
        })
        .collect()
}

/// write ARM assembly file into new file
fn write_asm_file(asm: &String, opts: &Opts, p: &PathBuf) -> PathBuf {
    let path = if opts.compile_only {
        opts.output
            .as_ref()
            .map(|s| PathBuf::from(s))
            .unwrap_or_else(|| new_output_asm(p, false))
    } else {
        new_output_asm(p, true)
    };

    let mut fout = File::create(&path).expect("failed to create file");
    fout.write_all(asm.as_bytes()).expect("wrote asm to file");
    info!("generate asm to {:?}", path);
    path
}

/// use system assembler (GNU as) to assemble asm code to object code
fn run_assembler(opts: &Opts, paths: &[PathBuf]) -> Vec<PathBuf> {
    info!("invoke assembler");
    paths
        .iter()
        .map(|p| {
            let output_path = if opts.complie_as_only {
                opts.output
                    .as_ref()
                    .map(|s| PathBuf::from(s))
                    .unwrap_or_else(|| new_output_obj(p, false))
            } else {
                new_output_obj(p, true)
            };

            let mut cmd = Command::new("/usr/bin/as");
            cmd.arg(p.as_os_str()).arg("-o").arg(&output_path);
            if opts.debug {
                cmd.arg("-v");
            }
            info!("{:?}", cmd);
            ensure_success(&mut cmd, "failed to assembler with asm files");
            output_path
        })
        .collect()
}

/// use system linker (GNU ld) to link object code to machine code (ELF)
fn run_linker(opts: &Opts, paths: &[PathBuf]) -> PathBuf {
    info!("invoke linker");

    let output_path = opts
        .output
        .as_ref()
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| new_output_executable(opts.files.first().unwrap()));

    // build command
    let mut cmd = Command::new("/usr/bin/ld");
    paths.iter().for_each(|p| {
        cmd.arg(p);
    });
    cmd.arg("-o").arg(&output_path);

    // run on macOS using Xcode ld
    if opts.target == TargetOs::MacOs {
        cmd
        .arg("-dynamic")
        .arg("-arch")
        .arg("arm64")
        .arg("-syslibroot")
        .arg("/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk")
        .arg("-lSystem");
    }

    // execute
    info!("{:?}", cmd);
    ensure_success(&mut cmd, "failed to run linker with object files");
    output_path
}

/// ensure command is succes
fn ensure_success(cmd: &mut Command, msg: &str) {
    match cmd.status() {
        Ok(status) if status.success() => (),
        Ok(_) => panic!("{}", msg),
        Err(e) => panic!("Failed to execute, error: {:?}", e),
    }
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

#[derive(Debug)]
struct Opts {
    files: Vec<PathBuf>,
    output: Option<String>,
    compile_only: bool,
    complie_as_only: bool,
    debug: bool,
    verbose: bool,
    target: TargetOs,
}

fn parse_opts() -> Opts {
    let app = create_arg_app();
    let args = app.get_matches();

    // load options
    let opts = Opts {
        target: TargetOs::current(),
        compile_only: args.is_present("arg-S"),
        complie_as_only: args.is_present("arg-c"),
        debug: args.is_present("debug"),
        verbose: args.is_present("verbose"),
        output: args.value_of("output").map(String::from),
        files: args
            .values_of("input")
            .unwrap()
            .map(|v| PathBuf::from(v))
            .collect(),
    };

    // validate
    if let Some(_) = opts.output {
        if opts.files.len() > 1 && (opts.compile_only || opts.complie_as_only) {
            panic!("can not specify '-o' with '-S' or '-c' when working with multiple input files");
        }
    }

    opts
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
