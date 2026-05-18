#![cfg(test)]

use super::*;

fn make_root_handle(fs: &mut Filesystem) -> DirectoryHandle<'_> {
    DirectoryHandle(&mut fs.root)
}

#[test]
fn file_handle_read_write_roundtrip() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    let mut file = root
        .create("notes.txt".to_string(), "hello".to_string())
        .expect("create file");
    assert_eq!(file.read(), "hello");

    file.write("updated".to_string());
    assert_eq!(file.read(), "updated");
}

#[test]
fn create_dir_and_nested_file() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    let mut src = root.create_dir("src".to_string()).expect("create dir");
    let file = src
        .create("main.rs".to_string(), "fn main() {}".to_string())
        .expect("create file");

    assert_eq!(file.read(), "fn main() {}");
}

#[test]
fn create_fails_if_name_exists() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    root.create("dup".to_string(), "one".to_string())
        .expect("first create");
    let err = root
        .create("dup".to_string(), "two".to_string())
        .expect_err("duplicate create should fail");

    assert!(matches!(err, FsError::AlreadyExists));
}

#[test]
fn create_dir_fails_if_name_exists() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    root.create_dir("dup".to_string())
        .expect("first create dir");
    let err = root
        .create_dir("dup".to_string())
        .expect_err("duplicate create dir should fail");

    assert!(matches!(err, FsError::AlreadyExists));
}

#[test]
fn find_file_and_directory_handles() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    root.create_dir("bin".to_string()).expect("create bin");
    root.create("Cargo.toml".to_string(), "[package]".to_string())
        .expect("create file");

    let file_path = Path::File("Cargo.toml".to_string());
    let dir_path = Path::File("bin".to_string());

    let file_handle = fs
        .find(&file_path)
        .expect("find file")
        .file_handle()
        .expect("file handle");
    assert_eq!(file_handle.read(), "[package]");

    let _dir_handle = fs
        .find(&dir_path)
        .expect("find dir")
        .dir_handle()
        .expect("dir handle");
}

#[test]
fn find_errors() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    root.create_dir("assets".to_string()).expect("create dir");
    root.create("README.md".to_string(), "doc".to_string())
        .expect("create file");

    let missing = Path::File("missing".to_string());
    let err = fs.find(&missing).expect_err("missing should error");
    assert!(matches!(err, FsError::NotFound));

    let not_a_file = Path::File("assets".to_string());
    let err = fs
        .find(&not_a_file)
        .expect("found")
        .file_handle()
        .expect_err("dir is not a file");
    assert!(matches!(err, FsError::NotAFile));

    let not_a_dir = Path::File("README.md".to_string());
    let err = fs
        .find(&not_a_dir)
        .expect("found")
        .dir_handle()
        .expect_err("file is not a dir");
    assert!(matches!(err, FsError::NotADirectory));
}

#[test]
fn display_filesystem_tree_ordered() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    root.create("Cargo.lock".to_string(), "".to_string())
        .expect("create file");
    root.create("Cargo.toml".to_string(), "".to_string())
        .expect("create file");

    let mut src = root.create_dir("src".to_string()).expect("create src");
    src.create("main.rs".to_string(), "".to_string())
        .expect("create main");

    let mut target = root
        .create_dir("target".to_string())
        .expect("create target");
    let mut debug = target
        .create_dir("debug".to_string())
        .expect("create debug");
    debug
        .create("twizzler".to_string(), "".to_string())
        .expect("create twizzler");
    let mut release = target
        .create_dir("release".to_string())
        .expect("create release");
    release
        .create("twizzler".to_string(), "".to_string())
        .expect("create twizzler");

    let printed = format!("{}", fs);
    let expected = "/
|- Cargo.lock
|- Cargo.toml
|- src/
   |- main.rs
|- target/
   |- debug/
      |- twizzler
   |- release/
      |- twizzler
";

    assert!(
        printed == expected,
        "printed: {}\nexpected: {}",
        printed,
        expected
    );
}

#[test]
fn display_path_from_iterator() {
    let parts = vec![
        "target".to_string(),
        "debug".to_string(),
        "twizzler".to_string(),
    ];
    let path: Path = parts.into_iter().collect();
    assert_eq!(format!("{}", path), "target/debug/twizzler");
}

#[test]
fn display_path_single_element() {
    let parts = vec!["only".to_string()];
    let path: Path = parts.into_iter().collect();
    assert_eq!(format!("{}", path), "only");
}

#[test]
fn display_path_multiple_elements() {
    let parts = vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
    ];
    let path: Path = parts.into_iter().collect();
    assert_eq!(format!("{}", path), "a/b/c/d");
}

#[test]
fn find_nested_file_with_full_path() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    let mut target = root
        .create_dir("target".to_string())
        .expect("create target");
    let mut release = target
        .create_dir("release".to_string())
        .expect("create release");
    release
        .create("twizzler".to_string(), "bin".to_string())
        .expect("create twizzler");

    let path = Path::Directory(
        "target".to_string(),
        Box::new(Path::Directory(
            "release".to_string(),
            Box::new(Path::File("twizzler".to_string())),
        )),
    );

    let handle = fs
        .find(&path)
        .expect("find nested")
        .file_handle()
        .expect("file handle");

    assert_eq!(handle.read(), "bin");
}

#[test]
fn find_nested_errors_when_not_a_directory() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    root.create("target".to_string(), "not a dir".to_string())
        .expect("create file");

    let path = Path::Directory(
        "target".to_string(),
        Box::new(Path::File("twizzler".to_string())),
    );

    let err = fs.find(&path).expect_err("should fail");
    assert!(matches!(err, FsError::NotADirectory));
}

#[test]
fn directory_handle_find_is_relative() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    let mut target = root
        .create_dir("target".to_string())
        .expect("create target");
    let mut debug = target
        .create_dir("debug".to_string())
        .expect("create debug");
    debug
        .create("twizzler".to_string(), "bin".to_string())
        .expect("create twizzler");

    let relative = Path::Directory(
        "debug".to_string(),
        Box::new(Path::File("twizzler".to_string())),
    );

    let handle = target
        .find(&relative)
        .expect("find relative")
        .file_handle()
        .expect("file handle");

    assert_eq!(handle.read(), "bin");
}

#[test]
fn write_persists_across_find() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    root.create("data.txt".to_string(), "old".to_string())
        .expect("create file");

    let path = Path::File("data.txt".to_string());
    {
        let mut handle = fs
            .find(&path)
            .expect("find file")
            .file_handle()
            .expect("file handle");
        handle.write("new".to_string());
    }

    let handle = fs
        .find(&path)
        .expect("find file")
        .file_handle()
        .expect("file handle");
    assert_eq!(handle.read(), "new");
}

#[test]
fn create_dir_then_create_file_same_name_fails() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    root.create_dir("conflict".to_string()).expect("create dir");

    let err = root
        .create("conflict".to_string(), "data".to_string())
        .expect_err("file with dir name should fail");

    assert!(matches!(err, FsError::AlreadyExists));
}

#[test]
fn create_file_then_create_dir_same_name_fails() {
    let mut fs = Filesystem::new();
    let mut root = make_root_handle(&mut fs);

    root.create("conflict".to_string(), "data".to_string())
        .expect("create file");

    let err = root
        .create_dir("conflict".to_string())
        .expect_err("dir with file name should fail");

    assert!(matches!(err, FsError::AlreadyExists));
}
