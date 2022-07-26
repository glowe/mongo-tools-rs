// These tests are ported almost directly from the original bsondump Go implementation

mod tests {
    use std::{
        io::{Read, Write},
        process::Stdio,
    };

    use rand::Rng;
    use tempfile::NamedTempFile;

    const SAMPLE_BSON: &[u8; 283] = include_bytes!("testdata/sample.bson");
    const SAMPLE_JSON: &[u8; 575] = include_bytes!("testdata/sample.json");

    #[test]
    fn from_stdin_to_stdout() {
        let mut child = test_bin::get_test_bin("bsondump")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn process");

        let mut stdin = child.stdin.take().expect("Failed to open stdin");

        std::thread::spawn(move || {
            stdin.write_all(SAMPLE_BSON).expect("Failed to write to stdin");
        });

        let output = child.wait_with_output().expect("Failed to read stdout");
        assert_eq!(&output.stdout, SAMPLE_JSON);
    }

    #[test]
    fn from_stdin_to_file() {
        let out_file = NamedTempFile::new().expect("Failed to create temporary out file");

        let mut child = test_bin::get_test_bin("bsondump")
            .args(["--outFile", out_file.path().to_str().expect("Failed get path")])
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to spawn process");

        let mut stdin = child.stdin.take().expect("Failed to open stdin");

        std::thread::spawn(move || {
            stdin.write_all(SAMPLE_BSON).expect("Failed to write to stdin");
        });

        child.wait().expect("Failed to write");

        let mut file = std::fs::File::open(out_file.path()).expect("Failed to open out file");
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf).expect("Failed to read out file");
        assert_eq!(buf, SAMPLE_JSON);
    }

    #[test]
    fn from_file_with_positional_argument_to_stdout() {
        let output = test_bin::get_test_bin("bsondump")
            .args(["tests/testdata/sample.bson"])
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to read process output");

        assert_eq!(&output.stdout, SAMPLE_JSON);
    }

    #[test]
    fn from_file_with_positional_argument_to_file() {
        let out_file = NamedTempFile::new().expect("Failed to create temporary out file");

        let mut child = test_bin::get_test_bin("bsondump")
            .args(["tests/testdata/sample.bson"])
            .args(["--outFile", out_file.path().to_str().expect("Failed get path")])
            .spawn()
            .expect("Failed to spawn process");

        child.wait().expect("Failed to wait for process");

        let mut file = std::fs::File::open(out_file.path()).expect("Failed to open out file");
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf).expect("Failed to read out file");
        assert_eq!(buf, SAMPLE_JSON);
    }

    const SIXTEEN_KB: usize = 16 * 1024;
    const MAX_SIZE: usize = (16 * 1024 * 1024) + SIXTEEN_KB;

    #[test]
    fn max_bson_size() {
        let output = run_with_bson_size(MAX_SIZE);
        assert!(output.status.success());
    }

    #[test]
    fn more_than_max_bson_size() {
        let output = run_with_bson_size(MAX_SIZE + 1);
        assert!(!output.status.success());
        assert!(String::from_utf8(output.stderr).unwrap().ends_with(
            "invalid BSONSize: 16793601 bytes is larger than than maximum of 16793600 bytes\n")
        );
    }

    fn run_with_bson_size(size: usize) -> std::process::Output {
        let binary_size: usize = size
            - SIXTEEN_KB // Subtract 16kb for the string field's data.
            - 4          // Subtract 4 bytes for the int32 at the head of the document that specifies its size.
            - 1          // Subtract 1 byte for the document's trailing NULL.
            - 2          // Subtract 2 bytes, one for for each byte that specifies the type of our two fields.
            - 1          // Subtract 1 byte for the binary field subtype specifier.
            - "name".len() // Subtract the length of the name key in the document.
            - "content".len() // Subtract the length of the content key in the document.
            - 2          // Subtract 2 bytes, one for each byte used as the trailing NULL in each of our two keys.
            - 8          // Subtract 8 bytes, 4 bytes for each of the int32 values specifying the length of our two fields.
            - 1; // Subtract 1 byte for the string field value's trailing NULL.

        let mut rng = rand::thread_rng();
        let bytes = (0..binary_size).map(|_| rng.gen()).collect::<Vec<u8>>();

        let mut doc = bson::Document::new();
        doc.insert("name", String::from("0123456789abcdef").repeat(1024));
        doc.insert("content", bson::Binary { subtype: bson::spec::BinarySubtype::Generic, bytes });

        let in_file = NamedTempFile::new().expect("Failed to create temporary file");
        doc.to_writer(&in_file).expect("Couldn't write to bson file");

        let out_file = NamedTempFile::new().expect("Failed to create temporary file");

        test_bin::get_test_bin("bsondump")
            .args([
                in_file.path().to_str().expect("Failed get path"),
                "--outFile",
                out_file.path().to_str().expect("Failed get path"),
            ])
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to collect process output")
    }
}
