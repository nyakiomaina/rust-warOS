// where to put the combatant struct
// wrapper around the loading, parsing, and interpreting of the wasm module as well as start()

use std::fmt;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use wasmi::{Error as InterpreterError, ImportsBuilder, Module, ModuleInstance, ModuleRef, RuntimeValue};

pub use crate::game::{GameState, Gameloop};
pub use crate::runtime::{Runtime, BOTINIT_NAME};

pub struct Combatant {}

impl Combatant {
    pub fn buffer_from_file(path: &str) -> Result<Vec<u8>> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::open(path)?;
        let mut wasm_buf = Vec::new();
        let _bytes_read = file.read_to_end(&mut wasm_buf)?;
        Ok(wasm_buf)
    }

    pub fn start(
        name: &str,
        buffer: Vec<u8>,
        game_state: Arc<crate::game::GameState>,
    ) -> JoinHandle<()> {
       let n = name.to_string();

        thread::spawn(move || {
            let module = Module::from_buffer(&buffer).unwrap();
            let mut runtime = runtime::Runtime::init(game_state, n.clone());
            let moduleref = Self::get_module_instance_from_module(&module).unwrap();
            let res = moduleref.invoke_export(BOTINIT_NAME, &[] [...], &mut runtime);
            println!("bot init loop exited for player {} - {:?}", n, res);
        })

    }

    pub fn get_module_instance_from_module(module: &Module) -> Result<ModuleRef> {
        let mut imports = ImportsBuilder::new();
        imports.push_resolver("env", &RuntimeModuleImportResolver);

        Ok(ModuleInstance::new(&module, &imports)
            .expect("Failed to instantiate module")
            .assert_no_start())
    }
}

//botengine error
#[derive(Debug)]
pub struct Error {
    kind: Kind,
}

impl HostError for Error {}

// standard error trait for the botengine error
impl std::error::Error for Error {
    fn description(&self) -> &str {
        "botengine error"
    }
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

// ensure botengine error can be string formatted
impl fmt::Display for Error {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            Kind::Interpreter(ref we) => fmt::Display::fmt(we, f),
            Kind::MiscFailure(ref s) => fmt::Display::fmt(s, f),
            Kind::IoError(ref s) => fmt::Display::fmt(s, f),
            Kind::ExportResolve(ref s) => fmt::Display::fmt(s, f),
        }
    }
}

// botengine error for I/O error
impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Error {
        Error {
            kind: Kind::IoError(source),
        }
    }
}

impl From<wasmi::Error> for Error {
    fn from(source: wasmi::Error) -> Error {
        Error {
            kind: Kind::InterpreterError(source),
        }
    }
}

// Indicates the kind of error that occurred.
#[derive(Debug)]
pub enum Kind {
    InterpreterError(wasmi::Error),
    IoError(std::io::Error),
    ExportResolve(String),
    MiscFailure(String),
}
// A Result where failure is a botengine error
pub type Result<T> = std::result::Result<T, Error>;
mod events;
mod game;
mod runtime;
mod game;