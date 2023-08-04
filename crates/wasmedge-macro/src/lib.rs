#![doc(
    html_logo_url = "https://github.com/cncf/artwork/blob/master/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.png?raw=true",
    html_favicon_url = "https://raw.githubusercontent.com/cncf/artwork/49169bdbc88a7ce3c4a722c641cc2d548bd5c340/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.svg"
)]

//! # Overview
//! The [wasmedge-macro](https://crates.io/crates/wasmedge-macro) crate defines a group of procedural macros used by both [wasmedge-sdk](https://crates.io/crates/wasmedge-sdk) and [wasmedge-sys](https://crates.io/crates/wasmedge-sys) crates.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, spanned::Spanned, FnArg, Item, Pat, PatType};

// ================== macros for wasmedge-sdk ==================

/// Declare a native function that will be used to create a host function instance.
#[proc_macro_attribute]
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
            &item_fn.sig.inputs.len()
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
        FnArg::Typed(PatType { ref ty, .. }) => match **ty {
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
                                                ref elem,
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
#[proc_macro_attribute]
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
            &item_fn.sig.inputs.len()
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
        FnArg::Typed(PatType { ref ty, .. }) => match **ty {
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
                                                ref elem,
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

#[doc(hidden)]
#[proc_macro_attribute]
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

#[doc(hidden)]
#[proc_macro_attribute]
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
                FnArg::Typed(PatType { ref ty, .. }) => match **ty {
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
                                                        ref elem,
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

#[doc(hidden)]
#[proc_macro_attribute]
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

#[doc(hidden)]
#[proc_macro_attribute]
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
