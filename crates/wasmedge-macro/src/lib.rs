#![doc(
    html_logo_url = "https://github.com/cncf/artwork/blob/master/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.png?raw=true",
    html_favicon_url = "https://raw.githubusercontent.com/cncf/artwork/49169bdbc88a7ce3c4a722c641cc2d548bd5c340/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.svg"
)]

//! # Overview
//! The [wasmedge-macro](https://crates.io/crates/wasmedge-macro) crate defines a group of procedural macros used by both [wasmedge-sdk](https://crates.io/crates/wasmedge-sdk) and [wasmedge-sys](https://crates.io/crates/wasmedge-sys) crates.

// -----------------------------------------------------------------------
// Deprecated: all six macros below (`host_function`,
// `async_host_function`, and the four `#[doc(hidden)]` `sys_*_host_function`
// macros) expand to the pre-0.14 host-function ABI, which was removed when
// `wasmedge-sdk` reached 0.14 / `wasmedge-sys` reached 0.19. Code using any
// of them does not compile against current crates. See each macro's doc
// comment for the modern replacement. They are marked `#[deprecated]` as of
// `wasmedge-macro` 0.7.0.
//
// Deprecation mechanics verified: rustc emits deprecated-attribute warnings
// at attribute use sites.
//
// Verified empirically (rustc/cargo 1.97.0) with a throwaway two-crate probe:
// a proc-macro crate exporting `#[deprecated] #[proc_macro_attribute] fn`,
// consumed via `#[that_attr]` on an item in a separate downstream crate.
// `cargo build` on the consumer emitted
// `warning: use of deprecated macro '<name>': <note>` pointing at the
// attribute's use site in the *consumer's* source — not just inside the
// macro-defining crate. This held with `#[deprecated]` placed both before
// and after `#[proc_macro_attribute]` on the function. A non-deprecated
// control macro produced no warning, isolating the effect to `#[deprecated]`
// itself. Conclusion: the standard `#[deprecated]` mechanism works as
// expected for `#[proc_macro_attribute]` fns — no ui-test fallback is
// needed.
// -----------------------------------------------------------------------

use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, Item, Pat, PatType, parse_macro_input, parse_quote, spanned::Spanned};

// ================== macros for wasmedge-sdk ==================

/// Declare a native function that will be used to create a host function instance.
///
/// # Deprecated: targets an API that no longer exists
///
/// This macro expands to the **pre-0.14** host-function ABI: a 3-argument free
/// function `(CallingFrame, Vec<WasmValue>, *mut c_void) -> Result<Vec<WasmValue>,
/// HostFuncError>`, wrapped around a generated call to `Caller::new(frame)`.
/// `Caller` no longer exists — it was removed when `wasmedge-sdk` reached 0.14 /
/// `wasmedge-sys` reached 0.19. `HostFuncError` still exists in
/// `wasmedge_types::error`, but it is unrelated to and unused by the current
/// host-function signature, which returns `Result<Vec<WasmValue>, CoreError>`
/// instead. **Any function annotated with `#[host_function]` fails to compile**
/// against current `wasmedge-sdk` (>= 0.14) or `wasmedge-sys` (>= 0.19).
///
/// This macro is marked `#[deprecated]` as of `wasmedge-macro` 0.7.0 (see the
/// verification note near the top of this file).
///
/// ## Modern replacement
///
/// Write the host function with today's signature directly — no macro needed —
/// and register it with `ImportObjectBuilder::with_func`:
///
/// ```rust,ignore
/// // This crate cannot depend on `wasmedge-sdk` (that would be circular), so
/// // this example is illustrative only. For a compiled, tested version, see
/// // `ImportObjectBuilder::with_func` in the wasmedge-sdk crate docs:
/// // https://docs.rs/wasmedge-sdk/latest/wasmedge_sdk/struct.ImportObjectBuilder.html
/// use wasmedge_sdk::{CallingFrame, ImportObjectBuilder, Instance, WasmValue, error::CoreError};
///
/// fn add(
///     _data: &mut (),
///     _inst: &mut Instance,
///     _frame: &mut CallingFrame,
///     args: Vec<WasmValue>,
/// ) -> Result<Vec<WasmValue>, CoreError> {
///     let (a, b) = (args[0].to_i32(), args[1].to_i32());
///     Ok(vec![WasmValue::from_i32(a + b)])
/// }
///
/// let mut builder = ImportObjectBuilder::new("env", ())?;
/// builder.with_func::<(i32, i32), i32>("add", add)?;
/// ```
#[proc_macro_attribute]
#[deprecated(
    since = "0.7.0",
    note = "targets the pre-0.14 host-function API removed from wasmedge-sdk; write the host function directly and register it with wasmedge_sdk::ImportObjectBuilder::with_func"
)]
pub fn host_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let body_ast = parse_macro_input!(item as Item);
    if let Item::Fn(item_fn) = body_ast {
        match expand_host_func(&item_fn) {
            Ok(token_stream) => token_stream.into(),
            Err(err) => err.to_compile_error().into(),
        }
    } else {
        TokenStream::new()
    }
}

fn expand_host_func(item_fn: &syn::ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    // extract T from Option<&mut T>
    let ret = match &item_fn.sig.inputs.len() {
        2 => expand_host_func_with_two_args(item_fn),
        3 => expand_host_func_with_three_args(item_fn),
        _ => panic!(
            "Invalid numbers of host function arguments: {}",
            item_fn.sig.inputs.len()
        ),
    };

    Ok(ret)
}

fn expand_host_func_with_two_args(item_fn: &syn::ItemFn) -> proc_macro2::TokenStream {
    // * define the signature of wrapper function
    // name of wrapper function
    let wrapper_fn_name_ident = item_fn.sig.ident.clone();
    let wrapper_fn_name_literal = wrapper_fn_name_ident.to_string();
    // arguments of wrapper function
    let wrapper_fn_inputs: syn::punctuated::Punctuated<FnArg, syn::token::Comma> = parse_quote!(
        frame: wasmedge_sdk::CallingFrame,
        args: Vec<wasmedge_sdk::WasmValue>,
        _data: *mut std::os::raw::c_void
    );
    // return type of wrapper function
    let wrapper_fn_return = item_fn.sig.output.clone();
    // visibility of wrapper function
    let wrapper_visibility = item_fn.vis.clone();

    // get the name of the first argument
    let (ident_first_arg, mutability) = match &item_fn.sig.inputs[0] {
        FnArg::Typed(PatType { pat, .. }) => match &**pat {
            Pat::Ident(pat_ident) => (pat_ident.ident.clone(), pat_ident.mutability),
            Pat::Wild(_) => (
                proc_macro2::Ident::new("_caller", proc_macro2::Span::call_site()),
                None,
            ),
            _ => panic!("The argument pattern of the first argument is not a simple ident"),
        },
        FnArg::Receiver(_) => panic!("The first argument is a receiver"),
    };

    // get the name of the second argument
    let ident_second_arg = match &item_fn.sig.inputs[1] {
        FnArg::Typed(PatType { pat, .. }) => match &**pat {
            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            Pat::Wild(_) => proc_macro2::Ident::new("_args", proc_macro2::Span::call_site()),
            _ => panic!("The argument pattern of the second argument is not a simple ident"),
        },
        FnArg::Receiver(_) => panic!("The second argument is a receiver"),
    };

    // * define the signature of inner function
    // name of inner function
    let inner_fn_name_literal = format!("inner_{wrapper_fn_name_literal}");
    let inner_fn_name_ident = syn::Ident::new(&inner_fn_name_literal, item_fn.sig.span());
    // arguments of inner function
    let inner_fn_inputs = item_fn.sig.inputs.clone();
    // return type of inner function
    let inner_fn_return = item_fn.sig.output.clone();
    // body of inner function
    let inner_fn_block = item_fn.block.clone();

    quote!(
        # wrapper_visibility fn #wrapper_fn_name_ident (#wrapper_fn_inputs) #wrapper_fn_return {
            // define inner function
            fn #inner_fn_name_ident (#inner_fn_inputs) #inner_fn_return {
                #inner_fn_block
            }

            // create a Caller instance
            let #mutability #ident_first_arg = Caller::new(frame);

            let #ident_second_arg = args;

            #inner_fn_name_ident(#ident_first_arg, #ident_second_arg)
        }
    )
}

fn expand_host_func_with_three_args(item_fn: &syn::ItemFn) -> proc_macro2::TokenStream {
    // * define the signature of wrapper function
    // name of wrapper function
    let wrapper_fn_name_ident = item_fn.sig.ident.clone();
    let wrapper_fn_name_literal = wrapper_fn_name_ident.to_string();
    // arguments of wrapper function
    let wrapper_fn_inputs: syn::punctuated::Punctuated<FnArg, syn::token::Comma> = parse_quote!(
        frame: wasmedge_sdk::CallingFrame,
        args: Vec<wasmedge_sdk::WasmValue>,
        data: *mut std::os::raw::c_void
    );
    // return type of wrapper function
    let wrapper_fn_return = item_fn.sig.output.clone();
    // visibility of wrapper function
    let wrapper_visibility = item_fn.vis.clone();

    // * define the signature of inner function
    // name of inner function
    let inner_fn_name_literal = format!("inner_{wrapper_fn_name_literal}");
    let inner_fn_name_ident = syn::Ident::new(&inner_fn_name_literal, item_fn.sig.span());
    // arguments of inner function
    let inner_fn_inputs = item_fn.sig.inputs.clone();
    // return type of inner function
    let inner_fn_return = item_fn.sig.output.clone();
    // body of inner function
    let inner_fn_block = item_fn.block.clone();

    // extract T from Option<&mut T>

    let data_arg = item_fn.sig.inputs.last().unwrap().clone();
    let ty_ptr = match &data_arg {
        FnArg::Typed(PatType { ty, .. }) => match **ty {
            syn::Type::Reference(syn::TypeReference { ref elem, .. }) => syn::TypePtr {
                star_token: parse_quote!(*),
                const_token: None,
                mutability: Some(parse_quote!(mut)),
                elem: elem.clone(),
            },
            syn::Type::Path(syn::TypePath { ref path, .. }) => match path.segments.last() {
                Some(segment) => {
                    let id = segment.ident.to_string();
                    match id == "Option" {
                        true => match segment.arguments {
                            syn::PathArguments::AngleBracketed(
                                syn::AngleBracketedGenericArguments { ref args, .. },
                            ) => {
                                let last_generic_arg = args.last();
                                match last_generic_arg {
                                    Some(arg) => match arg {
                                        syn::GenericArgument::Type(ty) => match ty {
                                            syn::Type::Reference(syn::TypeReference {
                                                elem,
                                                ..
                                            }) => syn::TypePtr {
                                                star_token: parse_quote!(*),
                                                const_token: None,
                                                mutability: Some(parse_quote!(mut)),
                                                elem: elem.clone(),
                                            },
                                            _ => panic!("Not found syn::Type::Reference"),
                                        },
                                        _ => {
                                            panic!("Not found syn::GenericArgument::Type")
                                        }
                                    },
                                    None => panic!("Not found the last GenericArgument"),
                                }
                            }
                            _ => panic!("Not found syn::PathArguments::AngleBracketed"),
                        },
                        false => panic!("Not found segment ident: Option"),
                    }
                }
                None => panic!("Not found path segments"),
            },
            _ => panic!("Unsupported syn::Type type"),
        },
        _ => panic!("Unsupported syn::FnArg type"),
    };

    // generate token stream
    quote!(
        # wrapper_visibility fn #wrapper_fn_name_ident (#wrapper_fn_inputs) #wrapper_fn_return {
            // define inner function
            fn #inner_fn_name_ident (#inner_fn_inputs) #inner_fn_return {
                #inner_fn_block
            }

            // create a Caller instance
            let caller = Caller::new(frame);

            let data = unsafe { &mut *(data as #ty_ptr) };

            #inner_fn_name_ident(caller, args, data)
        }
    )
}

/// Declare a native async function that will be used to create an async host function instance.
///
/// # Deprecated: targets an API that no longer exists
///
/// Like `host_function`, this macro expands to the **pre-0.14** ABI: a
/// 3-argument wrapper built around a generated `Caller::new(frame)` call,
/// returning `Box<dyn Future<Output = Result<Vec<WasmValue>, HostFuncError>> +
/// Send>`. `Caller` no longer exists — it was removed when `wasmedge-sdk`
/// reached 0.14 / `wasmedge-sys` reached 0.19. `HostFuncError` still exists in
/// `wasmedge_types::error`, but it is unrelated to and unused by the current
/// host-function signature, which returns `Result<Vec<WasmValue>, CoreError>`
/// instead. **Any function annotated with `#[async_host_function]` fails to
/// compile** against current `wasmedge-sdk` (>= 0.14) or `wasmedge-sys` (>=
/// 0.19).
///
/// This macro is marked `#[deprecated]` as of `wasmedge-macro` 0.7.0 (see the
/// verification note near the top of this file).
///
/// ## Modern replacement
///
/// Write the async host function with today's signature directly — no macro
/// needed — and register it with the async `ImportObjectBuilder::with_func`
/// (`wasmedge_sdk::r#async::import`, requires the `async` feature):
///
/// ```rust,ignore
/// // This crate cannot depend on `wasmedge-sdk` (that would be circular), so
/// // this example is illustrative only. For a compiled, tested version, see
/// // `ImportObjectBuilder::with_func` in the wasmedge-sdk crate docs:
/// // https://docs.rs/wasmedge-sdk/latest/wasmedge_sdk/r#async/import/struct.ImportObjectBuilder.html
/// use wasmedge_sdk::{
///     CallingFrame, WasmValue,
///     error::CoreError,
///     r#async::{AsyncInstance, import::ImportObjectBuilder},
/// };
///
/// fn add(
///     _data: &mut (),
///     _inst: &mut AsyncInstance,
///     _frame: &mut CallingFrame,
///     args: Vec<WasmValue>,
/// ) -> Box<dyn std::future::Future<Output = Result<Vec<WasmValue>, CoreError>> + Send> {
///     Box::new(async move {
///         let (a, b) = (args[0].to_i32(), args[1].to_i32());
///         Ok(vec![WasmValue::from_i32(a + b)])
///     })
/// }
///
/// let mut builder = ImportObjectBuilder::new("env", ())?;
/// builder.with_func::<(i32, i32), i32>("add", add)?;
/// ```
#[proc_macro_attribute]
#[deprecated(
    since = "0.7.0",
    note = "targets the pre-0.14 host-function API removed from wasmedge-sdk; write the host function directly and register it with wasmedge_sdk::ImportObjectBuilder::with_func"
)]
pub fn async_host_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let body_ast = parse_macro_input!(item as Item);
    if let Item::Fn(item_fn) = body_ast {
        if item_fn.sig.asyncness.is_none() {
            panic!("The function must be async");
        }

        match expand_async_host_func(&item_fn) {
            Ok(token_stream) => token_stream.into(),
            Err(err) => err.to_compile_error().into(),
        }
    } else {
        TokenStream::new()
    }
}

fn expand_async_host_func(item_fn: &syn::ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    // extract T from Option<&mut T>
    let ret = match &item_fn.sig.inputs.len() {
        2 => expand_async_host_func_with_two_args(item_fn),
        3 => expand_async_host_func_with_three_args(item_fn),
        _ => panic!(
            "Invalid numbers of host function arguments: {}",
            item_fn.sig.inputs.len()
        ),
    };

    Ok(ret)
}

fn expand_async_host_func_with_two_args(item_fn: &syn::ItemFn) -> proc_macro2::TokenStream {
    // * define the signature of wrapper function
    // name of wrapper function
    let wrapper_fn_name_ident = item_fn.sig.ident.clone();
    // arguments of wrapper function
    let wrapper_fn_inputs: syn::punctuated::Punctuated<FnArg, syn::token::Comma> = parse_quote!(
        frame: wasmedge_sdk::CallingFrame,
        args: Vec<wasmedge_sdk::WasmValue>,
        _data: *mut std::ffi::c_void
    );

    // visibility of wrapper function
    let wrapper_visibility = item_fn.vis.clone();

    // get the name of the first argument
    let (ident_first_arg, mutability) = match &item_fn.sig.inputs[0] {
        FnArg::Typed(PatType { pat, .. }) => match &**pat {
            Pat::Ident(pat_ident) => (pat_ident.ident.clone(), pat_ident.mutability),
            Pat::Wild(_) => (
                proc_macro2::Ident::new("_caller", proc_macro2::Span::call_site()),
                None,
            ),
            _ => panic!("The argument pattern of the first argument is not a simple ident"),
        },
        FnArg::Receiver(_) => panic!("The first argument is a receiver"),
    };

    // get the name of the second argument
    let ident_second_arg = match &item_fn.sig.inputs[1] {
        FnArg::Typed(PatType { pat, .. }) => match &**pat {
            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            Pat::Wild(_) => proc_macro2::Ident::new("_args", proc_macro2::Span::call_site()),
            _ => panic!("The argument pattern of the second argument is not a simple ident"),
        },
        FnArg::Receiver(_) => panic!("The second argument is a receiver"),
    };

    // func body
    let fn_block = item_fn.block.clone();

    quote!(
        #wrapper_visibility fn #wrapper_fn_name_ident (#wrapper_fn_inputs) -> Box<(dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send)> {

            // create a Caller instance
            let #mutability #ident_first_arg = Caller::new(frame);

            let #ident_second_arg = args;

            Box::new(async move {
                #fn_block
            })
        }
    )
}

fn expand_async_host_func_with_three_args(item_fn: &syn::ItemFn) -> proc_macro2::TokenStream {
    // * define the signature of wrapper function
    // name of wrapper function
    let wrapper_fn_name_ident = item_fn.sig.ident.clone();
    // arguments of wrapper function
    let wrapper_fn_inputs: syn::punctuated::Punctuated<FnArg, syn::token::Comma> = parse_quote!(
        frame: wasmedge_sdk::CallingFrame,
        args: Vec<wasmedge_sdk::WasmValue>,
        data: *mut std::ffi::c_void
    );
    // visibility of wrapper function
    let wrapper_visibility = item_fn.vis.clone();

    // get the name of the first argument
    let (ident_first_arg, mutability) = match &item_fn.sig.inputs[0] {
        FnArg::Typed(PatType { pat, .. }) => match &**pat {
            Pat::Ident(pat_ident) => (pat_ident.ident.clone(), pat_ident.mutability),
            Pat::Wild(_) => (
                proc_macro2::Ident::new("_caller", proc_macro2::Span::call_site()),
                None,
            ),
            _ => panic!("The argument pattern of the first argument is not a simple ident"),
        },
        FnArg::Receiver(_) => panic!("The first argument is a receiver"),
    };

    // get the name of the second argument
    let ident_second_arg = match &item_fn.sig.inputs[1] {
        FnArg::Typed(PatType { pat, .. }) => match &**pat {
            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            Pat::Wild(_) => proc_macro2::Ident::new("_args", proc_macro2::Span::call_site()),
            _ => panic!("The argument pattern of the second argument is not a simple ident"),
        },
        FnArg::Receiver(_) => panic!("The second argument is a receiver"),
    };

    // get the name of the third argument
    let ident_third_arg = match &item_fn.sig.inputs[2] {
        FnArg::Typed(PatType { pat, .. }) => match &**pat {
            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            Pat::Wild(_) => proc_macro2::Ident::new("_data", proc_macro2::Span::call_site()),
            _ => panic!("The argument pattern of the third argument is not a simple ident"),
        },
        FnArg::Receiver(_) => panic!("The third argument is a receiver"),
    };

    // get the type of the third argument
    let data_arg = item_fn.sig.inputs.last().unwrap().clone();
    let ty_third_arg = match &data_arg {
        FnArg::Typed(PatType { ty, .. }) => match **ty {
            syn::Type::Reference(syn::TypeReference { ref elem, .. }) => syn::TypePtr {
                star_token: parse_quote!(*),
                const_token: None,
                mutability: Some(parse_quote!(mut)),
                elem: elem.clone(),
            },
            syn::Type::Path(syn::TypePath { ref path, .. }) => match path.segments.last() {
                Some(segment) => {
                    let id = segment.ident.to_string();
                    match id == "Option" {
                        true => match segment.arguments {
                            syn::PathArguments::AngleBracketed(
                                syn::AngleBracketedGenericArguments { ref args, .. },
                            ) => {
                                let last_generic_arg = args.last();
                                match last_generic_arg {
                                    Some(arg) => match arg {
                                        syn::GenericArgument::Type(ty) => match ty {
                                            syn::Type::Reference(syn::TypeReference {
                                                elem,
                                                ..
                                            }) => syn::TypePtr {
                                                star_token: parse_quote!(*),
                                                const_token: None,
                                                mutability: Some(parse_quote!(mut)),
                                                elem: elem.clone(),
                                            },
                                            _ => panic!("Not found syn::Type::Reference"),
                                        },
                                        _ => {
                                            panic!("Not found syn::GenericArgument::Type")
                                        }
                                    },
                                    None => panic!("Not found the last GenericArgument"),
                                }
                            }
                            _ => panic!("Not found syn::PathArguments::AngleBracketed"),
                        },
                        false => panic!("Not found segment ident: Option"),
                    }
                }
                None => panic!("Not found path segments"),
            },
            _ => panic!("Unsupported syn::Type type"),
        },
        _ => panic!("Unsupported syn::FnArg type"),
    };

    // func body
    let fn_block = item_fn.block.clone();

    quote!(
        #wrapper_visibility fn #wrapper_fn_name_ident (#wrapper_fn_inputs) -> Box<(dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send)> {

            // create a Caller instance
            let #mutability #ident_first_arg = Caller::new(frame);

            let #ident_second_arg = args;

            // host context data
            let #ident_third_arg = unsafe { &mut *(data as #ty_third_arg) };

            Box::new(async move {
                #fn_block
            })
        }
    )
}

// ================== macros for wasmedge-sys ==================

/// Internal helper macro for `wasmedge-sys` async host functions.
///
/// # Deprecated: targets an API that no longer exists
///
/// Hidden from public docs, and — as of this writing — not used by any crate
/// in this workspace: `crates/wasmedge-sys/Cargo.toml` does not even depend
/// on `wasmedge-macro`. It expands the annotated function's body into
/// `Box<dyn Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send>`,
/// the pre-0.14 async host-function return convention. `HostFuncError` still
/// exists in `wasmedge_types::error`, but it is unrelated to and unused by
/// the current `AsyncFn<Data>` signature, which returns
/// `Result<Vec<WasmValue>, CoreError>` instead — so **any function annotated
/// with `#[sys_async_host_function]` fails to compile** against current
/// `wasmedge-sys` (>= 0.19).
///
/// Marked `#[deprecated]` as of `wasmedge-macro` 0.7.0 (see the verification
/// note near the top of this file).
///
/// ## Modern replacement
///
/// Write the function directly with today's `AsyncFn<Data>` shape — no macro
/// needed — and register it with `wasmedge_sys::r#async::function::AsyncFunction::create_async_func`
/// plus `AsyncImportObject::add_async_func` (or, for most users working
/// through the high-level crate, `wasmedge_sdk`'s async `ImportObjectBuilder::with_func`):
///
/// ```rust,ignore
/// // This crate cannot depend on `wasmedge-sys` (that would be circular), so
/// // this example is illustrative only. For a compiled, tested version, see
/// // `wasmedge-sys`'s `r#async::function::AsyncFn` and `AsyncImportObject` docs.
/// use wasmedge_sys::{
///     CallingFrame, WasmValue,
///     r#async::module::AsyncInstance,
/// };
/// use wasmedge_types::error::CoreError;
///
/// fn add(
///     _data: &mut (),
///     _inst: &mut AsyncInstance,
///     _frame: &mut CallingFrame,
///     args: Vec<WasmValue>,
/// ) -> Box<dyn std::future::Future<Output = Result<Vec<WasmValue>, CoreError>> + Send> {
///     Box::new(async move {
///         let (a, b) = (args[0].to_i32(), args[1].to_i32());
///         Ok(vec![WasmValue::from_i32(a + b)])
///     })
/// }
/// ```
#[doc(hidden)]
#[proc_macro_attribute]
#[deprecated(
    since = "0.7.0",
    note = "targets the pre-0.14 host-function API removed from wasmedge-sys; create it with wasmedge_sys::Function::create_sync_func and register via ImportModule::add_func"
)]
pub fn sys_async_host_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let body_ast = parse_macro_input!(item as Item);
    if let Item::Fn(item_fn) = body_ast {
        if item_fn.sig.asyncness.is_none() {
            panic!("The function must be async");
        }

        match sys_expand_async_host_func(&item_fn) {
            Ok(token_stream) => token_stream.into(),
            Err(err) => err.to_compile_error().into(),
        }
    } else {
        TokenStream::new()
    }
}

fn sys_expand_async_host_func(item_fn: &syn::ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    // extract T from Option<&mut T>
    let ret = match &item_fn.sig.inputs.len() {
        2 => sys_expand_async_host_func_with_two_args(item_fn),
        3 => sys_expand_async_host_func_with_three_args(item_fn),
        _ => panic!("Invalid numbers of host function arguments"),
    };

    Ok(ret)
}

fn sys_expand_async_host_func_with_two_args(item_fn: &syn::ItemFn) -> proc_macro2::TokenStream {
    // function name
    let fn_name_ident = &item_fn.sig.ident;

    // function visibility
    let fn_visibility = &item_fn.vis;

    // generic types
    let fn_generics = &item_fn.sig.generics;

    // arguments
    let mut fn_inputs = item_fn.sig.inputs.clone();
    fn_inputs.push(parse_quote!(_: *mut std::os::raw::c_void));

    // function body
    let fn_block = &item_fn.block;

    quote!(
        #fn_visibility fn #fn_name_ident #fn_generics (#fn_inputs) -> Box<(dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send)> {
            Box::new(async move {
                #fn_block
            })
        }
    )
}

fn sys_expand_async_host_func_with_three_args(item_fn: &syn::ItemFn) -> proc_macro2::TokenStream {
    // function name
    let fn_name_ident = &item_fn.sig.ident;

    // function visibility
    let fn_visibility = &item_fn.vis;

    // generic types
    let fn_generics = &item_fn.sig.generics;

    // arguments
    let fn_inputs = &item_fn.sig.inputs;

    // function body
    let fn_block = &item_fn.block;

    quote!(
        #fn_visibility fn #fn_name_ident #fn_generics (#fn_inputs) -> Box<(dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send)> {
            Box::new(async move {
                #fn_block
            })
        }
    )
}

/// Internal helper macro for `wasmedge-sys` sync host functions.
///
/// # Deprecated: targets an API that no longer exists
///
/// Hidden from public docs, and — as of this writing — not used by any crate
/// in this workspace: `crates/wasmedge-sys/Cargo.toml` does not even depend
/// on `wasmedge-macro`. It generates a wrapper whose data argument is threaded
/// as a positional `*mut c_void` — the pre-0.14 shape — rather than as the
/// leading `&mut Data` parameter of today's
/// `SyncFn<Data> = fn(&mut Data, &mut Instance, &mut CallingFrame, Vec<WasmValue>)
/// -> Result<Vec<WasmValue>, CoreError>`. A function written against the old
/// `HostFuncError`-based convention this macro assumes **fails to compile**
/// against current `wasmedge-sys` (>= 0.19), which uses
/// `wasmedge_types::error::CoreError` instead.
///
/// Marked `#[deprecated]` as of `wasmedge-macro` 0.7.0 (see the verification
/// note near the top of this file).
///
/// ## Modern replacement
///
/// Write the function directly with today's `SyncFn<Data>` shape — no macro
/// needed — and register it with `wasmedge_sys::Function::create_sync_func`
/// plus `ImportModule::add_func` (or, for most users working through the
/// high-level crate, `wasmedge_sdk::ImportObjectBuilder::with_func`):
///
/// ```rust,ignore
/// // This crate cannot depend on `wasmedge-sys` (that would be circular), so
/// // this example is illustrative only. For a compiled, tested version, see
/// // `wasmedge-sys`'s `SyncFn` and `ImportModule` docs.
/// use wasmedge_sys::{CallingFrame, Instance, WasmValue};
/// use wasmedge_types::error::CoreError;
///
/// fn add(
///     _data: &mut (),
///     _inst: &mut Instance,
///     _frame: &mut CallingFrame,
///     args: Vec<WasmValue>,
/// ) -> Result<Vec<WasmValue>, CoreError> {
///     let (a, b) = (args[0].to_i32(), args[1].to_i32());
///     Ok(vec![WasmValue::from_i32(a + b)])
/// }
/// ```
#[doc(hidden)]
#[proc_macro_attribute]
#[deprecated(
    since = "0.7.0",
    note = "targets the pre-0.14 host-function API removed from wasmedge-sys; create it with wasmedge_sys::Function::create_sync_func and register via ImportModule::add_func"
)]
pub fn sys_host_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let body_ast = parse_macro_input!(item as Item);
    if let Item::Fn(item_fn) = body_ast {
        match sys_expand_host_func_new(&item_fn) {
            Ok(token_stream) => token_stream.into(),
            Err(err) => err.to_compile_error().into(),
        }
    } else {
        TokenStream::new()
    }
}

fn sys_expand_host_func_new(item_fn: &syn::ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    // * define the signature of wrapper function
    // name of wrapper function
    let wrapper_fn_name_ident = item_fn.sig.ident.clone();
    let wrapper_fn_name_literal = wrapper_fn_name_ident.to_string();
    // return type of wrapper function
    let wrapper_fn_return = item_fn.sig.output.clone();
    // visibility of wrapper function
    let wrapper_fn_visibility = item_fn.vis.clone();

    // * define the signature of inner function
    // name of inner function
    let inner_fn_name_literal = format!("inner_{wrapper_fn_name_literal}");
    let inner_fn_name_ident = syn::Ident::new(&inner_fn_name_literal, item_fn.sig.span());
    // arguments of inner function
    let inner_fn_inputs = item_fn.sig.inputs.clone();
    // return type of inner function
    let inner_fn_return = item_fn.sig.output.clone();
    // body of inner function
    let inner_fn_block = item_fn.block.clone();

    // extract the identities of the first two arguments
    let arg1 = match &item_fn.sig.inputs[0] {
        FnArg::Typed(PatType { pat, .. }) => match &**pat {
            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            Pat::Wild(_) => proc_macro2::Ident::new("_", proc_macro2::Span::call_site()),
            _ => panic!("argument pattern is not a simple ident"),
        },
        FnArg::Receiver(_) => panic!("argument is a receiver"),
    };
    let arg2 = match &item_fn.sig.inputs[1] {
        FnArg::Typed(PatType { pat, .. }) => match &**pat {
            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            Pat::Wild(_) => proc_macro2::Ident::new("_", proc_macro2::Span::call_site()),
            _ => panic!("argument pattern is not a simple ident"),
        },
        FnArg::Receiver(_) => panic!("argument is a receiver"),
    };

    // extract T from Option<&mut T>
    let ret = match item_fn.sig.inputs.len() {
        2 => {
            // insert the third argument
            // let wrapper_fn_inputs = item_fn.sig.inputs.clone();
            let mut wrapper_fn_inputs = item_fn.sig.inputs.clone();
            wrapper_fn_inputs.push(parse_quote!(_data: *mut std::os::raw::c_void));

            quote!(
                #wrapper_fn_visibility fn #wrapper_fn_name_ident (#wrapper_fn_inputs) #wrapper_fn_return {
                    // define inner function
                    fn #inner_fn_name_ident (#inner_fn_inputs) #inner_fn_return {
                        #inner_fn_block
                    }

                    #inner_fn_name_ident(#arg1, #arg2)
                }
            )
        }
        3 => {
            let data_arg = item_fn.sig.inputs.last().unwrap().clone();
            let ty_ptr = match &data_arg {
                FnArg::Typed(PatType { ty, .. }) => match **ty {
                    syn::Type::Reference(syn::TypeReference { ref elem, .. }) => syn::TypePtr {
                        star_token: parse_quote!(*),
                        const_token: None,
                        mutability: Some(parse_quote!(mut)),
                        elem: elem.clone(),
                    },
                    syn::Type::Path(syn::TypePath { ref path, .. }) => match path.segments.last() {
                        Some(segment) => {
                            let id = segment.ident.to_string();
                            match id == "Option" {
                                true => match segment.arguments {
                                    syn::PathArguments::AngleBracketed(
                                        syn::AngleBracketedGenericArguments { ref args, .. },
                                    ) => {
                                        let last_generic_arg = args.last();
                                        match last_generic_arg {
                                            Some(arg) => match arg {
                                                syn::GenericArgument::Type(ty) => match ty {
                                                    syn::Type::Reference(syn::TypeReference {
                                                        elem,
                                                        ..
                                                    }) => syn::TypePtr {
                                                        star_token: parse_quote!(*),
                                                        const_token: None,
                                                        mutability: Some(parse_quote!(mut)),
                                                        elem: elem.clone(),
                                                    },
                                                    _ => panic!("Not found syn::Type::Reference"),
                                                },
                                                _ => {
                                                    panic!("Not found syn::GenericArgument::Type")
                                                }
                                            },
                                            None => panic!("Not found the last GenericArgument"),
                                        }
                                    }
                                    _ => panic!("Not found syn::PathArguments::AngleBracketed"),
                                },
                                false => panic!("Not found segment ident: Option"),
                            }
                        }
                        None => panic!("Not found path segments"),
                    },
                    _ => panic!("Unsupported syn::Type type"),
                },
                _ => panic!("Unsupported syn::FnArg type"),
            };

            // inputs of wrapper function
            let mut wrapper_fn_inputs = item_fn.sig.inputs.clone();
            wrapper_fn_inputs.pop();
            wrapper_fn_inputs.push(parse_quote!(data: *mut std::os::raw::c_void));

            // generate token stream
            quote!(
                #wrapper_fn_visibility fn #wrapper_fn_name_ident (#wrapper_fn_inputs) #wrapper_fn_return {
                    // define inner function
                    fn #inner_fn_name_ident (#inner_fn_inputs) #inner_fn_return {
                        #inner_fn_block
                    }

                    let data = unsafe { &mut *(data as #ty_ptr) };

                    #inner_fn_name_ident(#arg1, #arg2, data)
                }
            )
        }
        _ => panic!("Invalid numbers of host function arguments"),
    };

    Ok(ret)
}

// ================== macros for wasmedge-sys wasi host functions ==================

/// Internal helper macro for `wasmedge-sys` WASI-flavored sync host functions.
///
/// # Deprecated: targets an API that no longer exists
///
/// Hidden from public docs, and — as of this writing — not used by any crate
/// in this workspace: `crates/wasmedge-sys/Cargo.toml` does not even depend
/// on `wasmedge-macro`. It requires exactly 3 arguments and re-emits the
/// function body largely unchanged, so it inherits whatever ABI the caller's
/// function signature declares — historically the pre-0.14,
/// `HostFuncError`-returning convention that this macro family targets.
/// `HostFuncError` still exists in `wasmedge_types::error`, but it is
/// unrelated to and unused by the current `SyncFn<Data>` signature, which
/// returns `Result<Vec<WasmValue>, CoreError>` instead — so a function
/// written against the old convention **fails to compile** against current
/// `wasmedge-sys` (>= 0.19).
///
/// Marked `#[deprecated]` as of `wasmedge-macro` 0.7.0 (see the verification
/// note near the top of this file).
///
/// ## Modern replacement
///
/// Write the function directly with today's `SyncFn<Data>` shape — no macro
/// needed — and register it with `wasmedge_sys::Function::create_sync_func`
/// plus `ImportModule::add_func`, or use the `WasiModule` builder for WASI
/// imports specifically:
///
/// ```rust,ignore
/// // This crate cannot depend on `wasmedge-sys` (that would be circular), so
/// // this example is illustrative only. For a compiled, tested version, see
/// // `wasmedge-sys`'s `SyncFn` and `WasiModule` docs.
/// use wasmedge_sys::{CallingFrame, Instance, WasmValue};
/// use wasmedge_types::error::CoreError;
///
/// fn wasi_like(
///     _data: &mut (),
///     _inst: &mut Instance,
///     _frame: &mut CallingFrame,
///     args: Vec<WasmValue>,
/// ) -> Result<Vec<WasmValue>, CoreError> {
///     Ok(args)
/// }
/// ```
#[doc(hidden)]
#[proc_macro_attribute]
#[deprecated(
    since = "0.7.0",
    note = "targets the pre-0.14 host-function API removed from wasmedge-sys; create it with wasmedge_sys::Function::create_sync_func and register via ImportModule::add_func"
)]
pub fn sys_wasi_host_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let body_ast = parse_macro_input!(item as Item);
    if let Item::Fn(item_fn) = body_ast {
        match sys_expand_wasi_host_func(&item_fn) {
            Ok(token_stream) => token_stream.into(),
            Err(err) => err.to_compile_error().into(),
        }
    } else {
        TokenStream::new()
    }
}

fn sys_expand_wasi_host_func(item_fn: &syn::ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    // * define the signature of wrapper function
    // name of wrapper function
    let fn_name_ident = &item_fn.sig.ident;
    // return type of wrapper function
    let fn_return = &item_fn.sig.output;
    // visibility of wrapper function
    let fn_visibility = &item_fn.vis;

    // extract T from Option<&mut T>
    let ret = match item_fn.sig.inputs.len() {
        3 => {
            let fn_generics = &item_fn.sig.generics;

            // inputs of wrapper function
            let fn_inputs = &item_fn.sig.inputs;

            let fn_block = item_fn.block.clone();

            quote!(
                #fn_visibility fn #fn_name_ident #fn_generics (#fn_inputs) #fn_return
                    #fn_block
            )
        }
        _ => panic!("Invalid numbers of host function arguments"),
    };

    Ok(ret)
}

/// Internal helper macro for `wasmedge-sys` WASI-flavored async host functions.
///
/// # Deprecated: targets an API that no longer exists
///
/// Hidden from public docs, and — as of this writing — not used by any crate
/// in this workspace: `crates/wasmedge-sys/Cargo.toml` does not even depend
/// on `wasmedge-macro`. It expands the annotated function's body into
/// `Box<dyn Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send>`,
/// the pre-0.14 async host-function return convention. `HostFuncError` still
/// exists in `wasmedge_types::error`, but it is unrelated to and unused by
/// the current `AsyncFn<Data>` signature, which returns
/// `Result<Vec<WasmValue>, CoreError>` instead — so **any function annotated
/// with `#[sys_async_wasi_host_function]` fails to compile** against current
/// `wasmedge-sys` (>= 0.19).
///
/// Marked `#[deprecated]` as of `wasmedge-macro` 0.7.0 (see the verification
/// note near the top of this file).
///
/// ## Modern replacement
///
/// Write the function directly with today's `AsyncFn<Data>` shape — no macro
/// needed — and register it with `wasmedge_sys::r#async::function::AsyncFunction::create_async_func`
/// plus `AsyncImportObject::add_async_func`, or use the async `WasiModule`
/// builder for WASI imports specifically:
///
/// ```rust,ignore
/// // This crate cannot depend on `wasmedge-sys` (that would be circular), so
/// // this example is illustrative only. For a compiled, tested version, see
/// // `wasmedge-sys`'s `r#async::function::AsyncFn` and `AsyncWasiModule` docs.
/// use wasmedge_sys::{
///     CallingFrame, WasmValue,
///     r#async::module::AsyncInstance,
/// };
/// use wasmedge_types::error::CoreError;
///
/// fn wasi_like(
///     _data: &mut (),
///     _inst: &mut AsyncInstance,
///     _frame: &mut CallingFrame,
///     args: Vec<WasmValue>,
/// ) -> Box<dyn std::future::Future<Output = Result<Vec<WasmValue>, CoreError>> + Send> {
///     Box::new(async move { Ok(args) })
/// }
/// ```
#[doc(hidden)]
#[proc_macro_attribute]
#[deprecated(
    since = "0.7.0",
    note = "targets the pre-0.14 host-function API removed from wasmedge-sys; create it with wasmedge_sys::Function::create_sync_func and register via ImportModule::add_func"
)]
pub fn sys_async_wasi_host_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let body_ast = parse_macro_input!(item as Item);
    if let Item::Fn(item_fn) = body_ast {
        if item_fn.sig.asyncness.is_none() {
            panic!("The function must be async");
        }

        match sys_expand_async_wasi_host_func(&item_fn) {
            Ok(token_stream) => token_stream.into(),
            Err(err) => err.to_compile_error().into(),
        }
    } else {
        TokenStream::new()
    }
}

fn sys_expand_async_wasi_host_func(item_fn: &syn::ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    // extract T from Option<&mut T>
    let ret = match &item_fn.sig.inputs.len() {
        2 => sys_expand_async_wasi_host_func_with_two_args(item_fn),
        3 => sys_expand_async_wasi_host_func_with_three_args(item_fn),
        _ => panic!("Invalid numbers of host function arguments"),
    };

    Ok(ret)
}

fn sys_expand_async_wasi_host_func_with_two_args(
    item_fn: &syn::ItemFn,
) -> proc_macro2::TokenStream {
    // function name
    let fn_name_ident = &item_fn.sig.ident;

    // function visibility
    let fn_visibility = &item_fn.vis;

    // generic types
    let fn_generics = &item_fn.sig.generics;

    // arguments
    let mut fn_inputs = item_fn.sig.inputs.clone();
    fn_inputs.push(parse_quote!(_: *mut std::os::raw::c_void));

    // function body
    let fn_block = &item_fn.block;

    quote!(
        #fn_visibility fn #fn_name_ident #fn_generics (#fn_inputs) -> Box<(dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send)> {
            Box::new(async move {
                #fn_block
            })
        }
    )
}

fn sys_expand_async_wasi_host_func_with_three_args(
    item_fn: &syn::ItemFn,
) -> proc_macro2::TokenStream {
    // function name
    let fn_name_ident = &item_fn.sig.ident;

    // function visibility
    let fn_visibility = &item_fn.vis;

    // generic types
    let fn_generics = &item_fn.sig.generics;

    // arguments
    let fn_inputs = &item_fn.sig.inputs;

    // function body
    let fn_block = &item_fn.block;

    quote!(
        #fn_visibility fn #fn_name_ident #fn_generics (#fn_inputs) -> Box<(dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send)> {
            Box::new(async move {
                #fn_block
            })
        }
    )
}
