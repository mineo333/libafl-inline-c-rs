# libafl-inline-c
`libafl-inline-c` is a fork of `inline-c` that is specifically targeted towards LibAFL usecases. It attempts to expand on the functionaility by providing new abilities such as cross-compilation as well as shared object/dll compilation.

All the features from [`inline-c`](https://docs.rs/inline-c/0.1.7/inline_c/) are still present. However, some additional features are noted below.

## Shared-object compilation 

Shared-object compilation  can be done by simply adding the `#inline_c_rs SHARED` to the top of the C code. One can then get the output file by running `output_path()` on the `Assert` returned by `assert_c` or `assert_cxx`. One may use this output path, along with a library such as libloading to load the library. 

Windows DLLs, Apple Dylibs, and Linux SOs are supported.

For example:

```rust
use libafl_inline_c::assert_cxx;


fn test_shared(){
    let assert = assert_cxx!{
        #inline_c_rs SHARED
        
        #include <stdint.h>
        #include <stdlib.h>
        #include <string>
        
        #ifdef _MSC_VER
          #include <windows.h>
        
        BOOL APIENTRY DllMain(HANDLE hModule, DWORD ul_reason_for_call,
                              LPVOID lpReserved) {
          return TRUE;
        }
        
          #define EXTERN __declspec(dllexport) extern "C"
        #else
          #define EXTERN
        extern "C" {
        #endif
        
        EXTERN int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
          // abort();
          return 0;
        }
        
        #ifndef _MSC_VER
        }
        #endif
    };

    println!("{:?}", assert.output_path());
}

```

The above will compile to a shared object and print out the output path. 

## Cross-compilation

Cross compilation can be done via the `TARGET` option. Simply put `#inline_c_rs TARGET: "<target>"` at the top of the C code. The `<target>` refers to the rustup target (i.e., `x86_64-pc-windows-gnu`). 

This will automatically compile the code to that target. For example:

```rust
use libafl_inline_c::assert_cxx;


fn test_cross(){
    let assert = assert_cxx!{
        #inline_c_rs SHARED
        #inline_c_rs TARGET: "x86_64-pc-windows-gnu"
        #include <stdint.h>
        #include <stdlib.h>
        #include <string>
        
        #ifdef _MSC_VER
          #include <windows.h>
        
        BOOL APIENTRY DllMain(HANDLE hModule, DWORD ul_reason_for_call,
                              LPVOID lpReserved) {
          return TRUE;
        }
        
          #define EXTERN __declspec(dllexport) extern "C"
        #else
          #define EXTERN
        extern "C" {
        #endif
        
        EXTERN int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
          // abort();
          return 0;
        }
        
        #ifndef _MSC_VER
        }
        #endif
    }

    println!("{}", assert.output_path());
}


```

The above will compile to a windows DLL using the mingw toolchain. 

## Macros

The macro functionality is expanded upon from inline-c. In addition to `#define`, macro conditionals are also supported including `#ifdef`, `#else`, `#elif`, and `#endif`. However, only single-line macros are supported.

## License

`BSD-3-Clause`, see `LICENSE.md`.
