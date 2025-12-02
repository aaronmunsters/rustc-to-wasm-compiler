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
fn debugging_info_included() {
    use wasmito_addr2line::Module;

    let partial_config = ConfigurationBuilder::init()
        .stack_size(StackSize::Unspecified)
        .filename(Filename::Unspecified)
        .profile(Profile::O0)
        .source(FAC_SOURCE.into());

    let debug_files_len_for = |debug, profile| {
        let debuggable = partial_config
            .clone()
            .debugging(debug)
            .profile(profile)
            .build();
        let debuggable = Compiler::compile(&debuggable).unwrap();
        let debuggable = Module::new(debuggable);
        let debug_files = debuggable.files().unwrap();
        debug_files.len()
    };

    assert_eq!(debug_files_len_for(Debugging::Enabled, Profile::O0), 3);
    assert_eq!(debug_files_len_for(Debugging::Disabled, Profile::O3), 0);
}

#[test]
fn optimizations_affect() {
    let partial_config = ConfigurationBuilder::init()
        .stack_size(StackSize::Unspecified)
        .debugging(Debugging::Enabled)
        .filename(Filename::Unspecified)
        .source(FAC_SOURCE.into());

    let module_len_for = |profile| {
        let config = partial_config.clone().profile(profile).build();
        let module = Compiler::compile(&config).unwrap();
        module.len()
    };

    let profiles = [Profile::O0, Profile::O1, Profile::O2, Profile::O3];
    let lengths = profiles.map(module_len_for);
    assert_ne!(lengths[0], lengths[1]);
    assert_ne!(lengths[1], lengths[2]);
    // assert_ne!(lengths[2], lengths[3]); // --> No visible difference between `Profile::O2` && `Profile::O3`
}

#[test]
fn stack_size_configuration_affect() {
    let partial_config = ConfigurationBuilder::init()
        .profile(Profile::O3)
        .debugging(Debugging::Disabled)
        .filename(Filename::Unspecified)
        .source(FAC_SOURCE.into());

    let module_len_for = |stacksize| {
        let config = partial_config.clone().stack_size(stacksize).build();
        let module = Compiler::compile(&config).unwrap();
        module.len()
    };

    let profiles = [StackSize::Unspecified, StackSize::Configured(2_u32.pow(4))];
    let lengths = profiles.map(module_len_for);
    assert_ne!(lengths[0], lengths[1]);
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
