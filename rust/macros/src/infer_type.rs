/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *  http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */
//! Mynewt Macro that infers the types in a Rust function
extern crate proc_macro;
use proc_macro::TokenStream;
//use proc_macro2::Span;
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::prelude::*,
    path::Path,
};
use rustc_serialize::json;
use quote::{
    quote, 
    //quote_spanned,
};
use syn::{
    parse_macro_input,
    Block,
    Expr,
};

/// Return the Mynewt API function declaration for the function named `fname`
fn get_decl(fname: &str) -> &str {
    match fname {
        "sensor::set_poll_rate_ms"          => "&Strn, u32",
        "sensor::mgr_find_next_bydevname"   => "&Strn, *mut sensor",
        "sensor::register_listener"         => "*mut sensor, sensor_listener",
        "new_sensor_listener"               => "sensor_type_t, sensor_data_func",
        _ => ""
    }
}

/// Given a Rust function definition, infer the placeholder types in the function
pub fn infer_type_internal(_attr: TokenStream, item: TokenStream) -> TokenStream {
    //  println!("attr: {:#?}", attr); println!("item: {:#?}", item);
    //  Parse the macro input as Rust function definition.
    let input: syn::ItemFn = parse_macro_input!(item as syn::ItemFn);
    //  println!("input: {:#?}", input);

    //  Process the Function Declaration
    //  e.g. `fn start_sensor_listener(sensor: _, sensor_type: _, poll_time: _) -> MynewtResult<()>`
    let decl = input.decl;
    //  println!("decl: {:#?}", decl);

    //  `fname` is function name e.g. `start_sensor_listener`
    let fname = input.ident.to_string();
    println!("fname: {:#?}", fname);

    //  For each parameter e.g. `sensor`, `sensor_type`, `poll_time`...
    let mut all_para: ParaMap = HashMap::new();
    for input in &decl.inputs {
        //  Mark each parameter for Type Inference.
        //  println!("input: {:#?}", input);
        match input {
            syn::FnArg::Captured(arg_captured) => {
                //  println!("arg_captured: {:#?}", arg_captured);
                let pat = &arg_captured.pat;
                //  println!("pat: {:#?}", pat);
                //  `para` is the name of the parameter e.g. `sensor`
                let para = quote!{ #pat }.to_string();
                println!("para: {:#?}", para);
                all_para.insert(Box::new(para), Box::new("_".to_string()));
            }
            _ => { assert!(false, "Unknown input"); }
        }
    }

    //  Infer the types from the Block of code inside the function.
    let block = input.block;
    infer_from_block(&mut all_para, &block);

    //  Now that the types have been inferred, generate the Rust function declaration with the inferred types.
    let mut new_inputs = decl.inputs.clone();
    let mut all_para_types: ParaTypeList = Vec::new();
    //  For each parameter e.g. `sensor`, `sensor_type`, `poll_time`...
    for input in &mut new_inputs {
        //  Set the inferred type of each parameter.
        //  println!("input: {:#?}", input);
        match input {
            syn::FnArg::Captured(arg_captured) => {
                //  `para` is the name of the parameter e.g. `sensor`
                //  Change the type of the argument.
                let pat = &arg_captured.pat;
                let para = quote!{ #pat }.to_string();
                let type_str = match all_para.get(&para) {
                    //  Return the type if it exists.
                    Some(type_str) => type_str,
                    //  If we can't infer the type, leave as `_` for now. Maybe another function will infer this type.
                    None => "_"
                };
                if type_str != "_" {
                    //  If the type exists, remember it.
                    let tokens = type_str.parse().unwrap();
                    arg_captured.ty =  parse_macro_input!(tokens as syn::Type);
                }
                //  Remember the parameter type globally e.g. `[sensor, &Strn]`
                let para_type: ParaType = vec![Box::new(para), Box::new(type_str.to_string())];
                all_para_types.push(para_type);                
            }
            _ => { assert!(false, "Unknown input"); }
        }
    }

    //  Add this function to the global declaration list.
    let mut all_funcs: FuncTypeMap = HashMap::new();
    all_funcs.insert(Box::new(fname), all_para_types);
    save_decls(&all_funcs);

    //  Combine the new Rust function definition with the old function body.
    let new_decl = syn::FnDecl {
        inputs: new_inputs,
        ..*decl
    };
    let output = syn::ItemFn {
        decl: Box::new(new_decl),
        block: block,
        ..input
    };
    //  Return the new Rust function definition to the Rust Compiler.
    let expanded = quote! {        
        #output
    };
    expanded.into()
}

/// Load the function declarations from a JSON file.
fn load_decls() {
    // Create a path to the desired file
    let path = Path::new("hello.txt");
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   why.description()),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display,
                                                   why.description()),
        Ok(_) => print!("{} contains:\n{}", display, s),
    };
    //  let decoded: TestStruct = json::decode(&encoded).unwrap();
}

/// Save the function declarations to a JSON file.
fn save_decls(all_funcs: &FuncTypeMap) {
    let encoded = json::encode(&all_funcs).unwrap();
    println!("save_decls: {:#?}", encoded);
    let path = Path::new("test.json");
    let display = path.display();
    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why.description()),
        Ok(file) => file,
    };
    match file.write_all(encoded.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why.description()),
        Ok(_) => println!("successfully wrote to {}", display),
    };
}

/// Represents a map of parameter names indexed to the parameter type.
type ParaMap = HashMap<Box<String>, Box<String>>;
/// Represents the name and type of a parameter e.g. `[sensor, &Strn]`
type ParaType = Vec<Box<String>>;
/// Represents a list of parameter names and types
type ParaTypeList = Vec<ParaType>;
/// Represents a function name indexed to the function's parameters
type FuncTypeMap = HashMap<Box<String>, ParaTypeList>;

/// Infer the types of the parameters in `all_para` recursively from the function call `call`
fn infer_from_call(all_para: &mut ParaMap, call: &syn::ExprCall) {
    //  println!("call: {:#?}", call);
    //  For each function call `ExprCall`...    
    //  If this function call `ExprCall.func` is for a Mynewt API...
    //  e.g. `sensor::set_poll_rate_ms(sensor, poll_time) ? ;`
    let func = &call.func;
    // println!("func: {:#?}", quote!{ #func }.to_string());

    //  `fname` looks like `sensor::set_poll_rate_ms`
    let fname = quote!{ #func }.to_string().replace(" ", "");
    println!("fname: {:#?}", fname);

    //  Fetch the Mynewt API function declaration
    //  e.g. `fn sensor::set_poll_rate_ms(&Strn, u32)`
    let decl_str = get_decl(&fname);
    if decl_str == "" { return };
    let decl_types: Vec<&str> = decl_str.split(",").collect();

    //  For each argument `arg` in function call `ExprCall.args` e.g. `sensor`, `poll_time`, ...
    let args = &call.args;
    for pos in 0 .. args.len() {
        let arg = &args[pos];
        let arg_str = quote!{ #arg }.to_string().replace(" ", "");
        let decl_type = &decl_types[pos].trim();
        //  println!("arg: {:#?}", arg);

        //  If argument `arg` is not in our list of parameters `all_para`, skip.

        //  Match the identifier `ident` in `arg` (e.g. `sensor`) with the corresponding Mynewt API 
        //  parameter type `decl_type` (e.g. `&Strn`).
        println!("{} has inferred type {}", arg_str, decl_type);
        all_para.insert(Box::new(arg_str), Box::new(decl_type.to_string()));

        //  Remember the inferred type of the identifier...
        //  `sensor` has inferred type `&Strn`
        //  `poll_time` has inferred type `u32`

        /* `arg` looks like:
            Path(
            ExprPath {
                attrs: [],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: [
                        PathSegment {
                            ident: Ident {
                                ident: "sensor",
                                span: #0 bytes(0..0),
                            },
                            arguments: None, ...
        */
    }
}

///  For macro call `coap!( ..., sensor_data )`, infer `sensor_data` as `&SensorValue`
fn infer_from_macro(all_para: &mut ParaMap, macro_expr: &syn::ExprMacro) {
    //  println!("macro: {:#?}", macro_expr);
    let mac = &macro_expr.mac;
    let path = &mac.path;
    let tts = &mac.tts;
    //  `path` looks like `coap`
    //  `tts` looks like `@ json { \"device\" : & device_id , sensor_data , }`
    println!("path: {:#?}", quote!{ #path }.to_string());
    println!("tts: {:#?}", quote!{ #tts }.to_string());
}

/// Infer the types of the parameters in `all_para` recursively from the code block `block`
fn infer_from_block(all_para: &mut ParaMap, block: &Block) {
    //  For each statement in the block...
    //  e.g. `sensor::set_poll_rate_ms(sensor, poll_time) ?`
    for stmt in &block.stmts {
        //  Look for the expression inside the statement and infer the types from the expression.
        //  println!("stmt: {:#?}", stmt);
        match stmt {
            //  `let x = ...`
            syn::Stmt::Local(local) => {
                if let Some((_eq, expr)) = &local.init {
                    infer_from_expr(all_para, &expr);
                }
            }
            //  `fname( ... )`
            syn::Stmt::Expr(expr) => { infer_from_expr(all_para, &expr); }
            //  `fname( ... );`
            syn::Stmt::Semi(expr, _semi) => { infer_from_expr(all_para, &expr); }
            //  Not interested in item definitions: `fn fname( ... ) { ... }`
            syn::Stmt::Item(_item) => {}
        };
        /* `stmt` looks like:
                Semi(
                    Try(
                        ExprTry {
                            attrs: [],
                            expr: Call(
                                ExprCall {
                                    attrs: [],
                                    func: Path(
                                        ExprPath {
                                            attrs: [],
                                            qself: None,
                                            path: Path {
                                                leading_colon: None,
                                                segments: [
                                                    PathSegment {
                                                        ident: Ident {
                                                            ident: "sensor",
                                                            span: #0 bytes(0..0),
                                                        },
                                                        arguments: None,
                                                    },
                                                    Colon2,
                                                    PathSegment {
                                                        ident: Ident {
                                                            ident: "set_poll_rate_ms",
                                                            span: #0 bytes(0..0),
                                                        },
                                                        arguments: None, ...
        */        
    }
}

/// Infer the types of the parameters in `all_para` recursively from the expression `expr`
fn infer_from_expr(all_para: &mut ParaMap, expr: &Expr) {
    //  println!("expr: {:#?}", expr);
    match expr {
        //  `fname( ... )`
        Expr::Call(expr) => { infer_from_call(all_para, &expr); }
        //  `... + ...`
        Expr::Binary(expr) => {
            infer_from_expr(all_para, &expr.left);
            infer_from_expr(all_para, &expr.right);
        }
        //  `- ...`
        Expr::Unary(expr) => {
            infer_from_expr(all_para, &expr.expr);            
        }
        //  `let x = ...`
        Expr::Let(expr) => {
            infer_from_expr(all_para, &expr.expr);            
        }
        //  `if cond { ... } else { ... }`
        Expr::If(expr) => {
            infer_from_expr(all_para, &expr.cond);
            infer_from_block(all_para, &expr.then_branch);
            if let Some((_else, expr)) = &expr.else_branch {
                infer_from_expr(all_para, &expr);
            }
        }
        //  `while cond { ... }`
        Expr::While(expr) => {
            infer_from_expr(all_para, &expr.cond);
            infer_from_block(all_para, &expr.body);
        }
        //  `for i in ... { ... }`
        Expr::ForLoop(expr) => {
            infer_from_expr(all_para, &expr.expr);
            infer_from_block(all_para, &expr.body);
        }
        //  `loop { ... }`
        Expr::Loop(expr) => {
            infer_from_block(all_para, &expr.body);

        }
        //  `( ... )`
        Expr::Paren(expr) => {
            infer_from_expr(all_para, &expr.expr);
        }
        //  `...`
        Expr::Group(expr) => {
            infer_from_expr(all_para, &expr.expr);
        }
        //  `fname( ... ) ?`
        Expr::Try(expr) => { 
            infer_from_expr(all_para, &expr.expr); 
        }
        //  For macro call `coap!( ..., sensor_data )`, infer `sensor_data` as `&SensorValue`
        Expr::Macro(expr) => {
            infer_from_macro(all_para, &expr);
        }

        //  TODO: Box, Array, MethodCall, Tuple, Match, Closure, Unsafe, Block, Assign, AssignOp

        //  Not interested: InPlace, Field, Index, Range, Path, Reference, Break, Continue, Return, Struct, Repeat, Async, TryBlock, Yield, Verbatim
        _ => {}
    };
}