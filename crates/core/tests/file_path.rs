use toql_core::file_path::FilePath;

#[test]
fn test_path_parents() {
    assert_eq!(&[""], FilePath::from("user").parents().collect())
}
