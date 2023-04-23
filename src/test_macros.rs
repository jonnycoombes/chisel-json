#[macro_export]
macro_rules! relative_file {
    ($f : expr) => {{
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        base.join($f)
    }};
}

#[macro_export]
macro_rules! file_from_relative_path {
    ($f : expr) => {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = base.join($f);
        let f = File::open(path).unwrap();
    };
}

#[macro_export]
macro_rules! chars_from_file {
    ($f : expr) => {{
        let f = File::open($f).unwrap();
        let mut reader = BufReader::new(f);
        let decoders = DecoderSelector::default();
        DecoderSelector::default().default_decoder(&mut reader)
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
