use std::fmt::Debug;

use rayon::prelude::*;
use rustc_to_wasm_compiler::Compiler;
use rustc_to_wasm_compiler::configuration::{
    Configuration, Debugging, Filename, Profile, StackSize,
};
use rustc_to_wasm_compiler::configuration_builder::ConfigurationBuilder;
use wasmtime::{WasmParams, WasmResults};

mod mocked_fs;

const PROFILE_OPTS: &[Profile; 4] = {
    use Profile::{O0, O1, O2, O3};
    &[O0, O1, O2, O3]
};

const STACK_SIZES: &[fn() -> StackSize; 2] = {
    use StackSize::{Configured, Unspecified};
    &[
        // User did not specify anything
        || Unspecified,
        // User specifies the stack size
        || Configured(32768),
    ]
};

const DEBUG_OPTS: &[Debugging; 2] = {
    use Debugging::{Disabled, Enabled};
    &[Enabled, Disabled]
};

const FILENAME_CONFIGS: &[fn() -> Filename] = {
    use Filename::{Configured, Unspecified};
    &[
        // User did not specify anything
        || Unspecified,
        // User specifies as source 'lib.rs'
        || Configured("lib.rs".into()),
    ]
};

#[test]
fn test_semver() -> anyhow::Result<()> {
    Compiler::version()?;
    Ok(())
}

const FAC_SOURCE: &str = include_str!("fac.rs_");

#[test]
fn test_different_variants() {
    FILENAME_CONFIGS.par_iter().for_each(|filename_config| {
        DEBUG_OPTS.par_iter().for_each(|debug_option| {
            STACK_SIZES.par_iter().for_each(|stack_size| {
                PROFILE_OPTS.par_iter().for_each(|profile_option| {
                    let config = ConfigurationBuilder::init()
                        .debugging(*debug_option)
                        .stack_size(stack_size())
                        .profile(*profile_option)
                        .source(FAC_SOURCE.into())
                        .filename(filename_config())
                        .build();

                    // Assert on the outcome
                    assert_outcome(&config, "fac", 5, &120).unwrap();
                });
            });
        });
    });
}

#[test]
fn recursive_input() -> anyhow::Result<()> {
    let source = r#"
        #![no_main]
        #![no_std]

        #[no_mangle]
        pub extern "C" fn fac(n: i32) -> i32 {
            if n == 0 {
                return 1;
            } else {
                return n * fac(n - 1);
            }
        }

        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }
    "#;

    let config = ConfigurationBuilder::init()
        .debugging(Debugging::Enabled)
        .stack_size(StackSize::Unspecified)
        .profile(Profile::O0)
        .source(source.into())
        .filename(Filename::Unspecified)
        .build();

    assert_outcome(&config, "fac", 5, &120)?;
    Ok(())
}

#[test]
fn unsafe_c_overflow() -> anyhow::Result<()> {
    let source = r#"
        #![no_main]
        #![no_std]

        #[no_mangle]
        pub extern "C" fn overflow(a: i32, b: i32) -> i32 {
            let (res, overflow) = a.overflowing_add(b);
            let _ = overflow; // overflow is ignored
            res
        }

        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }
    "#;

    let config = ConfigurationBuilder::init()
        .debugging(Debugging::Disabled)
        .stack_size(StackSize::Unspecified)
        .profile(Profile::O3)
        .source(source.into())
        .filename(Filename::Unspecified)
        .build();

    assert_outcome(&config, "overflow", (i32::MAX, 1), &i32::MIN)?;
    Ok(())
}

#[test]
fn failing_compilation() {
    let config = ConfigurationBuilder::init()
        .debugging(Debugging::Disabled)
        .stack_size(StackSize::Unspecified)
        .profile(Profile::O3)
        .source("no rust source code".into())
        .filename(Filename::Unspecified)
        .build();

    assert!(
        Compiler::compile(&config)
            .is_err_and(|err| matches!(err, rustc_to_wasm_compiler::error::Error::Unsuccesful(_)))
    );
}

fn assert_outcome<Params: WasmParams, Results: WasmResults + Eq + Debug>(
    config: &Configuration,
    exported: &str,
    params: Params,
    results: &Results,
) -> anyhow::Result<()> {
    let output = {
        /* Compiling Rust source code into Wasm */
        Compiler::compile(config)?
    };

    {
        /* Running the module */
        use wasmtime::{Engine, Instance, Module, Store};
        let engine = Engine::default();
        let module = Module::from_binary(&engine, &output)?;
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;
        let outcome = instance
            .get_typed_func::<Params, Results>(&mut store, exported)?
            .call(&mut store, params)?;

        assert_eq!(outcome, *results);
    };

    Ok(())
}

#[test]
fn configuration_settings() {
    let config = ConfigurationBuilder::init()
        .debugging(Debugging::Disabled)
        .stack_size(StackSize::Unspecified)
        .profile(Profile::O0)
        .source("hi there!".into())
        .filename(Filename::Unspecified)
        .build();

    assert_eq!(config.debugging(), &Debugging::Disabled);
    assert_eq!(config.profile(), &Profile::O0);
    assert_eq!(config.source(), "hi there!");
}
