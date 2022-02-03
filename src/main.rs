#![allow(dead_code)]

mod shader_importer;
mod windowed_run;
mod windowless_run;

fn main() {
    // windowed_run::main();
    windowless_run::main();
}
