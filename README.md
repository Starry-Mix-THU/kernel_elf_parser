# elf_parser_rs

[![Crates.io](https://img.shields.io/crates/v/elf_parser_rs)](https://crates.io/crates/elf_parser_rs)
[![Docs.rs](https://docs.rs/elf_parser_rs/badge.svg)](https://docs.rs/elf_parser_rs)
[![CI](https://github.com/Azure-stars/elf_parser_rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Azure-stars/elf_parser_rs/actions/workflows/ci.yml)

A lightweight ELF parser written in Rust, providing assistance for loading applications into the kernel.

It reads the data of the ELF file, and generates Sections, Relocations, Segments and so on.

It also generate a layout of the user stack according to the given user parameters and environment variables,which will be 
used for loading a given application into the physical memory of the kernel.

## Examples

```rust
use std::collections::BTreeMap;
let args: Vec<String> = vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()];
let envs: Vec<String> = vec!["LOG=file".to_string()];
let auxv = BTreeMap::new();
// The highest address of the user stack.
let ustack_end = 0x4000_0000;
let ustack_size = 0x1_0000;
let ustack_start = ustack_end - ustack_size;

let stack_data =
    elf_parser_rs::app_stack_region(&args, &envs, &auxv, ustack_start.into(), ustack_size);
assert_eq!(stack_data[0..8], [3, 0, 0, 0, 0, 0, 0, 0]);

// uspace.map_alloc(ustack_start, ustack_size, MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER)?;

let ustack_pointer = ustack_end - stack_data.len();

// Copy the stack data to the user stack.
// After initialization, the stack layout is as follows: <https://articles.manugarg.com/aboutelfauxiliaryvectors.html>
// unsafe {
//     core::ptr::copy_nonoverlapping(
//         stack_data.as_ptr(),
//         ustack_pointer as *mut u8,
//         stack_data.len(),
//     );
// }
println!("ustack_pointer: {}", ustack_pointer);
```