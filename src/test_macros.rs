#[macro_export]
macro_rules! reader_from_relative_file {
    ($f : expr) => {{
        let path = env::current_dir().unwrap().join($f);
        let f = File::open(path).unwrap();
        BufReader::new(f)
    }};
}

#[macro_export]
macro_rules! reader_from_bytes {
    ($b : expr) => {{
        let buffer: &[u8] = $b.as_bytes();
        BufReader::new(buffer)
    }};
}

#[macro_export]
macro_rules! lines_from_relative_file {
    ($f : expr) => {{
        let path = env::current_dir().unwrap().join($f);
        let f = File::open(path).unwrap();
        BufReader::new(f).lines()
    }};
}
