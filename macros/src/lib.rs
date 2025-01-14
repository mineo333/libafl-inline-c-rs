//! Please see the `inline-c` crate to learn more.
#![feature(proc_macro_span)]
use proc_macro2::TokenStream;
use quote::quote;

/// Execute a C program and return a `Result` of
/// `inline_c::Assert`. See examples inside the `inline-c` crate.
#[proc_macro]
pub fn assert_c(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    let input_as_string = reconstruct(input);

    quote!(
        libafl_inline_c::run(libafl_inline_c::Language::C, #input_as_string).map_err(|e| panic!("{}", e)).unwrap()
    )
    .into()
}

/// Execute a C++ program and return a `Result` of
/// `inline_c::Assert`. See examples inside the `inline-c` crate.
#[proc_macro]
pub fn assert_cxx(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    let input_as_string = reconstruct(input);

    quote!(
        libafl_inline_c::run(libafl_inline_c::Language::Cxx, #input_as_string).map_err(|e| panic!("{}", e)).unwrap()
    )
    .into()
}

fn reconstruct(input: TokenStream) -> String {
    use proc_macro2::{Delimiter, Spacing, TokenTree::*};

    let mut output = String::new();
    let mut iterator = input.into_iter().peekable();

    loop {
        match iterator.next() {
            Some(Punct(token)) => {
                let token_value = token.as_char();

                match token_value {
                    '#' => {
                        output.push('\n');
                        output.push(token_value);

                        match iterator.peek() {
                            // #include …
                            Some(Ident(include)) if *include == "include" => {
                                iterator.next();

                                match iterator.next() {
                                    // #include <…>
                                    Some(Punct(punct)) => {
                                        if punct.as_char() != '<' {
                                            panic!(
                                                "Invalid opening token after `#include`, received `{:?}`.",
                                                token
                                            )
                                        }

                                        output.push_str("include <");

                                        loop {
                                            match iterator.next() {
                                                Some(Punct(punct)) => {
                                                    let punct = punct.as_char();

                                                    if punct == '>' {
                                                        break;
                                                    }

                                                    output.push(punct)
                                                }

                                                Some(Ident(ident)) => {
                                                    output.push_str(&ident.to_string())
                                                }

                                                token => panic!(
                                                    "Invalid token in `#include` value, with `{:?}`.",
                                                    token
                                                ),
                                            }
                                        }

                                        output.push('>');
                                        output.push('\n');
                                    }

                                    // #include "…"
                                    Some(Literal(literal)) => {
                                        output.push_str("include ");
                                        output.push_str(&literal.to_string());
                                        output.push('\n');
                                    }

                                    Some(token) => panic!(
                                        "Invalid opening token after `#include`, received `{:?}`.",
                                        token
                                    ),

                                    None => panic!("`#include` must be followed by `<` or `\"`."),
                                }
                            }
                            // #define, only available on nightly.
                            Some(Ident(define)) if *define == "define" || *define == "ifdef" || *define == "else" || *define == "endif" || *define == "elif" => {
                                #[cfg(not(nightly))]
                                panic!(
                                    "`#define` in C is only supported in `libafl_inline_c` with Rust nightly"
                                );

                                #[cfg(nightly)]
                                {
                                    let current_line = define.span().unwrap().start().line();
                                    output.push_str(&define.to_string());
                                    iterator.next();

                                    output.push(' ');

                                    loop {
                                        match iterator.peek() {
                                            Some(item) => {
                                                if item.span().unwrap().start().line()
                                                    == current_line
                                                {
                                                    output.push_str(&item.to_string());
                                                    output.push(' ');
                                                    iterator.next();
                                                } else {
                                                    output.push('\n');
                                                    break;
                                                }
                                            }

                                            None => break,
                                        }
                                    }
                                }
                            }

                            _ => (),
                        }
                    }

                    ';' => {
                        output.push(token_value);
                        output.push('\n');
                    }

                    _ => {
                        output.push(token_value);

                        if token.spacing() == Spacing::Alone {
                            output.push(' ');
                        }
                    }
                }
            }

            Some(Ident(ident)) => {
                output.push_str(&ident.to_string());
                output.push(' ');
            }

            Some(Group(group)) => {
                let group_output = reconstruct(group.stream());

                match group.delimiter() {
                    Delimiter::Parenthesis => {
                        output.push('(');
                        output.push_str(&group_output);
                        output.push(')');
                    }

                    Delimiter::Brace => {
                        output.push('{');
                        output.push('\n');
                        output.push_str(&group_output);
                        output.push('\n');
                        output.push('}');
                        output.push('\n');
                    }

                    Delimiter::Bracket => {
                        output.push('[');
                        output.push_str(&group_output);
                        output.push(']');
                    }

                    Delimiter::None => {
                        output.push_str(&group_output);
                    }
                }
            }

            Some(token) => {
                output.push_str(&token.to_string());
                //this is a special case because on windows targetting compilers it expects a space between extern "C" [return type]
                if token.to_string() == "\"C\"" {
                    output.push(' ');
                }
            }

            None => break,
        }
    }

    output
}
