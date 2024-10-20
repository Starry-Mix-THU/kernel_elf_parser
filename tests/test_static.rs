#[test]
fn test_elf_parser() {
    use memory_addr::VirtAddr;
    // A simple elf file compiled by the x86_64-linux-musl-gcc.
    let elf_bytes = include_bytes!("elf_static");
    // Ensure the alignment of the byte array
    let mut aligned_elf_bytes = unsafe {
        let ptr = elf_bytes.as_ptr() as *mut u8;
        std::slice::from_raw_parts_mut(ptr, elf_bytes.len())
    }
    .to_vec();
    if aligned_elf_bytes.len() % 16 != 0 {
        let padding = vec![0u8; 16 - aligned_elf_bytes.len() % 16];
        aligned_elf_bytes.extend(padding);
    }
    let elf =
        xmas_elf::ElfFile::new(aligned_elf_bytes.as_slice()).expect("Failed to read elf file");

    let elf_base_addr = 0x1000;
    let base_addr = elf_parser_rs::elf_base_addr(&elf, elf_base_addr).unwrap();
    assert_eq!(base_addr, 0);

    let segments = elf_parser_rs::elf_segments(&elf, base_addr);
    assert_eq!(segments.len(), 4);
    let mut last_start = VirtAddr::from_usize(0);
    for segment in segments.iter() {
        // start vaddr should be sorted
        assert!(segment.vaddr > last_start);
        last_start = segment.vaddr;
    }
    assert_eq!(segments[0].vaddr, VirtAddr::from_usize(0x400000));

    test_ustack(&elf, base_addr);
}

fn test_ustack(elf: &xmas_elf::ElfFile, base_addr: usize) {
    let auxv = elf_parser_rs::auxv_vector(&elf, base_addr);
    const AT_PHENT: u8 = 4;
    let phent = auxv.get(&AT_PHENT).unwrap();
    assert_eq!(*phent, 56);

    let args: Vec<String> = vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()];
    let envs: Vec<String> = vec!["LOG=file".to_string()];

    // The highest address of the user stack.
    let ustack_end = 0x4000_0000;
    let ustack_size = 0x2_0000;
    let ustack_bottom = ustack_end - ustack_size;

    let stack_data =
        elf_parser_rs::app_stack_region(&args, &envs, &auxv, ustack_bottom.into(), ustack_size);
    // The first 8 bytes of the stack is the number of arguments.
    assert_eq!(stack_data[0..8], [3, 0, 0, 0, 0, 0, 0, 0]);
}
