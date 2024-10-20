#[test]
fn test_elf_parser() {
    use memory_addr::VirtAddr;
    // A simple elf file compiled by the gcc 11.4.
    let elf_bytes = include_bytes!("elf_dynamic");
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
    assert_eq!(base_addr, elf_base_addr);

    let segments = elf_parser_rs::elf_segments(&elf, base_addr);
    assert_eq!(segments.len(), 4);
    for segment in segments.iter() {
        println!("{:?} {:?}", segment.vaddr, segment.flags);
    }
    assert_eq!(segments[0].vaddr, VirtAddr::from_usize(0x1000));
}
