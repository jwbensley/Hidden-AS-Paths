use bgpkit_parser::BgpkitParser;

/// Print a specific record/entry from an MRT file
pub fn print_entry(index: &u32, filename: &String) {
    let parser = BgpkitParser::new(filename.as_str())
        .unwrap_or_else(|_| panic!("Unable to parse {}", filename));
    let mut count: u32 = 0;

    for mrt_entry in parser.into_record_iter() {
        if count != *index {
            count += 1;
            continue;
        }

        println!("{:#?}", mrt_entry);
        return;
    }
}
