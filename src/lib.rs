//!`libafl-inline-c` is a fork of `inline-c` that is specifically targeted towards LibAFL usecases. It attempts to expand on the functionaility by providing new abilities such as cross-compilation as well as shared object/dll compilation.
//!
//!All the features from [`inline-c`](https://docs.rs/inline-c/0.1.7/inline_c/) are still present. However, some additional features are noted below.
//!
//!## Shared-object compilation 
//!
//!Shared-object compilation  can be done by simply adding the `#inline_c_rs SHARED` to the top of the C code. One can then get the output file by running [`Assert::output_path`] on the [`Assert`] returned by [`assert_c`] or [`assert_cxx`]. One may use this output path, along with a library such as libloading to load the library. 
//!
//!Windows DLLs, Apple Dylibs, and Linux SOs are supported.
//!
//!For example:
//!
//!```rust
//!use libafl_inline_c::assert_cxx;
//!
//!fn test_shared(){
//!    let assert = assert_cxx!{
//!        #inline_c_rs SHARED
//!        
//!        #include <stdint.h>
//!        #include <stdlib.h>
//!        #include <string>
//!        
//!        #ifdef _MSC_VER
//!          #include <windows.h>
//!        
//!        BOOL APIENTRY DllMain(HANDLE hModule, DWORD ul_reason_for_call,
//!                              LPVOID lpReserved) {
//!          return TRUE;
//!        }
//!        
//!          #define EXTERN __declspec(dllexport) extern "C"
//!        #else
//!          #define EXTERN
//!        extern "C" {
//!        #endif
//!        
//!        EXTERN int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
//!          // abort();
//!          return 0;
//!        }
//!        
//!        #ifndef _MSC_VER
//!        }
//!        #endif
//!    }
//!
//!    println!("{}", assert.output_path());
//!}
//!```
//!
//!
//!
//!
//!The above will compile to a shared object and print out the output path. 
//!
//!## Cross-compilation
//!
//!Cross compilation can be done via the `TARGET` option. Simply put `#inline_c_rs TARGET: "<target>"` at the top of the C code. The `<target>` refers to the rustup target (i.e., `x86_64-pc-windows-gnu`). 
//!
//!This will automatically compile the code to that target. For example:
//!
//!```rust
//!use libafl_inline_c::assert_cxx;
//!
//!
//!fn test_cross(){
//!    let assert = assert_cxx!{
//!        #inline_c_rs SHARED
//!        #inline_c_rs TARGET: "x86_64-pc-windows-gnu"
//!        #include <stdint.h>
//!        #include <stdlib.h>
//!        #include <string>
//!        
//!        #ifdef _MSC_VER
//!          #include <windows.h>
//!        
//!        BOOL APIENTRY DllMain(HANDLE hModule, DWORD ul_reason_for_call,
//!                              LPVOID lpReserved) {
//!          return TRUE;
//!        }
//!        
//!          #define EXTERN __declspec(dllexport) extern "C"
//!        #else
//!          #define EXTERN
//!        extern "C" {
//!        #endif
//!        
//!        EXTERN int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
//!          // abort();
//!          return 0;
//!        }
//!        
//!        #ifndef _MSC_VER
//!        }
//!        #endif
//!    }
//!
//!    println!("{}", assert.output_path());
//!}
//!```
//!The above will compile to a windows DLL using the mingw toolchain. 
//!
//!## Macros
//!
//!The macro functionality is expanded upon from inline-c. In addition to `#define`, macro conditionals are also supported including `#ifdef`, `#else`, `#elif`, and `#endif`. However, only single-line macros are supported.




mod assert;
mod run;

pub use crate::run::{run, Language};
pub use assert::Assert;
pub use libafl_inline_c_macro::{assert_c, assert_cxx};
pub mod predicates {
    //! Re-export the prelude of the `predicates` crate, which is useful for assertions.
    //!
    //! # Example
    //!
    //! An end of line on all systems are represented by the `\n`
    //! character, except on Windows where it is `\r\n`. Even if C
    //! writes `\n`, it will be translated into `\r\n`, so we need to
    //! normalize this. This is where the `predicates` crate can be
    //! helpful.
    //!
    //! ```rust
    //! use inline_c::{assert_c, predicates::*};
    //!
    //! fn test_predicates() {
    //!     (assert_c! {
    //!         #include <stdio.h>
    //!
    //!         int main() {
    //!             printf("Hello, World!\n");
    //!
    //!             return 0;
    //!         }
    //!     })
    //!     .success()
    //!     .stdout(predicate::eq("Hello, World!\n").normalize());
    //! }
    //!
    //! # fn main() { test_predicates() }
    //! ```

    pub use predicates::prelude::*;
}

#[cfg(test)]
mod tests {
    use super::predicates::*;
    use super::*;
    use crate as inline_c;
    use std::env::{remove_var, set_var};

    #[test]
    fn test_c_macro() {
        (assert_c! {
            int main() {
                int x = 1;
                int y = 2;

                return x + y;
            }
        })
        .failure()
        .code(3);
    }

    #[test]
    fn test_c_macro_with_include() {
        (assert_c! {
            #include <stdio.h>

            int main() {
                printf("Hello, World!\n");

                return 0;
            }
        })
        .success()
        .stdout(predicate::eq("Hello, World!\n").normalize());
    }

    #[test]
    fn test_c_macro_with_env_vars_inlined() {
        set_var("INLINE_C_RS_CFLAGS", "-D_CRT_SECURE_NO_WARNINGS");

        (assert_c! {
            // Those are env variables.
            #inline_c_rs FOO: "bar baz qux"
            #inline_c_rs HELLO: "World!"

            #include <stdio.h>
            #include <stdlib.h>

            int main() {
                const char* foo = getenv("FOO");
                const char* hello = getenv("HELLO");

                if (NULL == foo || NULL == hello) {
                    return 1;
                }

                printf("FOO is set to `%s`\n", foo);
                printf("HELLO is set to `%s`\n", hello);

                return 0;
            }
        })
        .success()
        .stdout(
            predicate::eq(
                "FOO is set to `bar baz qux`\n\
                HELLO is set to `World!`\n",
            )
            .normalize(),
        );

        remove_var("INLINE_C_RS_CFLAGS");
    }

    #[test]
    fn test_c_macro_with_env_vars_from_env_vars() {
        // Define env vars through env vars.
        set_var("INLINE_C_RS_FOO", "bar baz qux");
        set_var("INLINE_C_RS_HELLO", "World!");
        set_var("INLINE_C_RS_CFLAGS", "-D_CRT_SECURE_NO_WARNINGS");

        (assert_c! {
            #include <stdio.h>
            #include <stdlib.h>

            int main() {
                const char* foo = getenv("FOO");
                const char* hello = getenv("HELLO");

                if (NULL == foo || NULL == hello) {
                    return 1;
                }

                printf("FOO is set to `%s`\n", foo);
                printf("HELLO is set to `%s`\n", hello);

                return 0;
            }
        })
        .success()
        .stdout(
            predicate::eq(
                "FOO is set to `bar baz qux`\n\
                HELLO is set to `World!`\n",
            )
            .normalize(),
        );

        remove_var("INLINE_C_RS_FOO");
        remove_var("INLINE_C_RS_HELLO");
        remove_var("INLINE_C_RS_CFLAGS");
    }

    /* #[cfg(nightly)]
    #[test]
    fn test_c_macro_with_define() {
        (assert_c! {
            #define sum(a, b) ((a) + (b))

            int main() {
                return !(sum(1, 2) == 3);
            }
        })
        .success();
    } */
}
