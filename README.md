# Rust to WebAssembly Compiler using `rustc`

Rust crate for compiling Rust source code to WebAssembly using `rustc`.

## Prerequisites

Requires `rustc` installed and in your PATH.

## Usage

```rust
use rustc_to_wasm_compiler::{Compiler, configuration_builder::ConfigurationBuilder};
use rustc_to_wasm_compiler::configuration::{Debugging, StackSize, Profile, Filename};

let c_source = r#"
    pub extern "C" fn add(a: i32, b: i32) -> i32 {
        a + b
    }
"#;

let config = ConfigurationBuilder::init()
    .source(c_source.into())
    .profile(Profile::O2)
    .debugging(Debugging::Disabled)
    .stack_size(StackSize::Unspecified)
    .filename(Filename::Unspecified)
    .build();

let wasm_bytes = Compiler::compile(&config)?;
```

## Configuration

**Profiles**: `O0`, `O1`, `O2`, `O3`  
**Debugging**: `Enabled`, `Disabled`
**StackSize**: `Unspecified`, `Configured<u32>`

## Exporting Rust Functions

Use the function definition `pub extern "C" fn f(...) -> ... { ... }` to make functions callable from WASM:

```c
pub extern "C" fn my_function(arg: i32) -> i32 {
    arg * 2
}
```

## Author

Aäron Munsters

## License

MIT
