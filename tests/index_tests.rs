use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use tiny_dc::index::DirectoryIndex;

#[test]
fn directory_index_z_returns_correct_result() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create temporary directory inside the temp directory
    let temp_test_dir = temp_dir.path().join("test_dir");
    std::fs::create_dir_all(&temp_test_dir).unwrap();

    // Create temporary directory inside the temp directory
    let temp_test_dir_other = temp_dir.path().join("test_dir_other");
    std::fs::create_dir_all(&temp_test_dir_other).unwrap();

    // Create temporary directory inside the temp directory
    let temp_other_dir = temp_dir.path().join("other_dir");
    std::fs::create_dir_all(&temp_other_dir).unwrap();

    let index_file_path = temp_dir.path().join(".tiny-dc");

    let mut file = File::create(&index_file_path).unwrap();

    let mock_data = vec![
        (temp_test_dir.to_str().unwrap(), 100, 100),
        (temp_test_dir_other.to_str().unwrap(), 150, 100),
        (temp_other_dir.to_str().unwrap(), 100, 100),
    ];

    for line in mock_data {
        writeln!(file, "{}|{}|{}\n", line.0, line.1, line.2).unwrap();
    }

    let mut directory_index = DirectoryIndex::try_from(index_file_path.clone()).unwrap();
    let result = directory_index.z(vec!["test".into()]).unwrap();

    assert_eq!(result, Some(temp_test_dir_other.to_str().unwrap().into()));
}

#[test]
fn directory_index_z_returns_correct_result_for_case_sensitive_query() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create temporary directory inside the temp directory
    let temp_test_dir = temp_dir.path().join("Test_Dir");
    std::fs::create_dir_all(&temp_test_dir).unwrap();

    // Create temporary directory inside the temp directory
    let temp_test_dir_other = temp_dir.path().join("test_dir_other");
    std::fs::create_dir_all(&temp_test_dir_other).unwrap();

    // Create temporary directory inside the temp directory
    let temp_other_dir = temp_dir.path().join("other_dir");
    std::fs::create_dir_all(&temp_other_dir).unwrap();

    let index_file_path = temp_dir.path().join(".tiny-dc");

    let mut file = File::create(&index_file_path).unwrap();

    let mock_data = vec![
        (temp_test_dir.to_str().unwrap(), 100, 100),
        (temp_test_dir_other.to_str().unwrap(), 150, 100),
        (temp_other_dir.to_str().unwrap(), 100, 100),
    ];

    for line in mock_data {
        writeln!(file, "{}|{}|{}\n", line.0, line.1, line.2).unwrap();
    }

    let mut directory_index = DirectoryIndex::try_from(index_file_path.clone()).unwrap();
    let result = directory_index.z(vec!["Test".into()]).unwrap();

    assert_eq!(result, Some(temp_test_dir.to_str().unwrap().into()));
}

#[test]
fn directory_index_z_returns_correct_result_for_multiple_queries() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create temporary directory inside the temp directory
    let temp_test_dir = temp_dir.path().join("test_dir");
    std::fs::create_dir_all(&temp_test_dir).unwrap();

    // Create temporary directory inside the temp directory
    let temp_test_dir_other = temp_dir.path().join("test_dir_other");
    std::fs::create_dir_all(&temp_test_dir_other).unwrap();

    // Create temporary directory inside the temp directory
    let temp_other_dir = temp_dir.path().join("other_dir");
    std::fs::create_dir_all(&temp_other_dir).unwrap();

    let index_file_path = temp_dir.path().join(".tiny-dc");

    let mut file = File::create(&index_file_path).unwrap();

    let mock_data = vec![
        (temp_test_dir.to_str().unwrap(), 100, 100),
        (temp_test_dir_other.to_str().unwrap(), 150, 100),
        (temp_other_dir.to_str().unwrap(), 100, 100),
    ];

    for line in mock_data {
        writeln!(file, "{}|{}|{}\n", line.0, line.1, line.2).unwrap();
    }

    let mut directory_index = DirectoryIndex::try_from(index_file_path.clone()).unwrap();
    let result = directory_index
        .z(vec!["other".into(), "test".into()])
        .unwrap();

    assert_eq!(result, Some(temp_test_dir_other.to_str().unwrap().into()));
}

#[test]
fn directory_index_z_returns_existing_path_only() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create temporary directory inside the temp directory
    let temp_test_dir = temp_dir.path().join("test_dir");
    std::fs::create_dir_all(&temp_test_dir).unwrap();

    // Create temporary directory inside the temp directory
    let temp_test_dir_other = temp_dir.path().join("test_dir_other");
    std::fs::create_dir_all(&temp_test_dir_other).unwrap();

    // Create temporary directory inside the temp directory
    let temp_other_dir = temp_dir.path().join("other_dir");
    std::fs::create_dir_all(&temp_other_dir).unwrap();

    let index_file_path = temp_dir.path().join(".tiny-dc");

    let mut file = File::create(&index_file_path).unwrap();

    let mock_data = vec![
        (temp_test_dir.to_str().unwrap(), 100, 100),
        (temp_test_dir_other.to_str().unwrap(), 150, 100),
        (temp_other_dir.to_str().unwrap(), 100, 100),
    ];

    // Delete the temp_test_dir_other directory to simulate a non-existing path.
    std::fs::remove_dir_all(&temp_test_dir_other).unwrap();

    for line in mock_data {
        writeln!(file, "{}|{}|{}\n", line.0, line.1, line.2).unwrap();
    }

    let mut directory_index = DirectoryIndex::try_from(index_file_path.clone()).unwrap();
    let result = directory_index.z(vec!["test".into()]).unwrap();

    assert_eq!(result, Some(temp_test_dir.to_str().unwrap().into()));
}

#[test]
fn directory_index_z_returns_none_for_no_match() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create temporary directory inside the temp directory
    let temp_test_dir = temp_dir.path().join("test_dir");
    std::fs::create_dir_all(&temp_test_dir).unwrap();

    // Create temporary directory inside the temp directory
    let temp_test_dir_other = temp_dir.path().join("test_dir_other");
    std::fs::create_dir_all(&temp_test_dir_other).unwrap();

    // Create temporary directory inside the temp directory
    let temp_other_dir = temp_dir.path().join("other_dir");
    std::fs::create_dir_all(&temp_other_dir).unwrap();

    let index_file_path = temp_dir.path().join(".tiny-dc");

    let mut file = File::create(&index_file_path).unwrap();

    let mock_data = vec![
        (temp_test_dir.to_str().unwrap(), 100, 100),
        (temp_test_dir_other.to_str().unwrap(), 150, 100),
        (temp_other_dir.to_str().unwrap(), 100, 100),
    ];

    for line in mock_data {
        writeln!(file, "{}|{}|{}\n", line.0, line.1, line.2).unwrap();
    }

    let mut directory_index = DirectoryIndex::try_from(index_file_path.clone()).unwrap();
    let result = directory_index.z(vec!["non-existent".into()]).unwrap();

    assert_eq!(result, None);
}

#[test]
fn directory_index_z_returns_correct_result_for_common_parent() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create a common parent directory.
    let common_parent = temp_dir.path().join("common_parent");
    std::fs::create_dir_all(&common_parent).unwrap();

    // Create two subdirectories inside the common parent.
    let child_dir = common_parent.join("child");
    std::fs::create_dir_all(&child_dir).unwrap();

    let nested_dir = child_dir.join("nested");
    std::fs::create_dir_all(&nested_dir).unwrap();

    // Write the paths to the index file.
    let index_file_path = temp_dir.path().join(".tiny-dc");
    let mut file = File::create(&index_file_path).unwrap();

    // Here we set up three entries. Even though the `child` and its nested directory have
    // higher rank values, we want the query for "common" to match the common parent directory.
    let mock_data = vec![
        (common_parent.to_str().unwrap(), 100, 100), // Common parent entry.
        (child_dir.to_str().unwrap(), 200, 200),     // Child entry with higher rank.
        (nested_dir.to_str().unwrap(), 300, 300),    // Nested entry with the highest rank.
    ];
    for line in mock_data {
        writeln!(file, "{}|{}|{}", line.0, line.1, line.2).unwrap();
    }

    // Load the index and query for the common parent.
    let mut directory_index = DirectoryIndex::try_from(index_file_path.clone()).unwrap();
    let result = directory_index.z(vec!["common".into()]).unwrap();

    // Assert that the common parent is returned even if a subdirectory has a higher rank.
    assert_eq!(result, Some(common_parent.to_str().unwrap().into()),);
}

#[test]
fn directory_index_z_returns_none_for_empty_index() {
    let temp_dir = tempfile::tempdir().unwrap();
    let index_file_path = temp_dir.path().join(".tiny-dc");

    let mut directory_index = DirectoryIndex::try_from(index_file_path.clone()).unwrap();
    let result = directory_index.z(vec!["nonexistent".into()]).unwrap();

    assert_eq!(result, None);
}

#[test]
fn directory_index_push_creates_index_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let index_file_path = temp_dir.path().join(".tiny-dc");

    // Create temporary directory inside the temp directory
    let temp_test_dir = temp_dir.path().join("test_dir");
    std::fs::create_dir_all(&temp_test_dir).unwrap();

    // Create a new DirectoryIndex and push an entry.
    let mut directory_index = DirectoryIndex::try_from(index_file_path.clone()).unwrap();
    directory_index.push(temp_test_dir.clone()).unwrap();

    // Check if the index file was created
    assert!(index_file_path.exists());

    // Check if the entry was added to the index
    let file = File::open(&index_file_path).unwrap();
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|line| line.unwrap()).collect();
    let line = &lines[0];
    let parts: Vec<&str> = line.split('|').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], temp_test_dir.to_str().unwrap());
}

fn get_index_file_lines(index_file_path: &PathBuf) -> Vec<String> {
    let file = File::open(index_file_path).unwrap();
    let reader = BufReader::new(file);
    reader.lines().map(|line| line.unwrap()).collect()
}

#[test]
fn directory_index_push_multiple_times_updates_entry_rank() {
    let temp_dir = tempfile::tempdir().unwrap();
    let index_file_path = temp_dir.path().join(".tiny-dc");

    // Create temporary directory inside the temp directory
    let temp_test_dir = temp_dir.path().join("test_dir");
    std::fs::create_dir_all(&temp_test_dir).unwrap();

    let mut directory_index = DirectoryIndex::try_from(index_file_path.clone()).unwrap();
    directory_index.push(temp_test_dir.clone()).unwrap();

    let lines = get_index_file_lines(&index_file_path);
    let line = &lines[0];
    let parts: Vec<&str> = line.split('|').collect();

    // Check if the entry was added to the index
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[1], "0");

    // Push the entry second time
    let mut directory_index = DirectoryIndex::try_from(index_file_path.clone()).unwrap();
    directory_index.push(temp_test_dir.clone()).unwrap();

    let lines = get_index_file_lines(&index_file_path);
    let line = &lines[0];
    let parts: Vec<&str> = line.split('|').collect();

    // Check if the entry was updated in the index
    assert_eq!(parts.len(), 3);
    // The rank should be updated to 1
    assert_eq!(parts[1], "1");

    // Push the entry third time
    let mut directory_index = DirectoryIndex::try_from(index_file_path.clone()).unwrap();
    directory_index.push(temp_test_dir.clone()).unwrap();

    let lines = get_index_file_lines(&index_file_path);
    let line = &lines[0];
    let parts: Vec<&str> = line.split('|').collect();

    // Check if the entry was updated in the index
    assert_eq!(parts.len(), 3);
    // The rank should be updated to 1.99
    assert_eq!(parts[1], "1.99");
}

#[test]
fn directory_index_push_non_existent_path_does_is_a_no_op() {
    let temp_dir = tempfile::tempdir().unwrap();
    let index_file_path = temp_dir.path().join(".tiny-dc");

    // Create a new DirectoryIndex and push a non-existent entry.
    let mut directory_index = DirectoryIndex::try_from(index_file_path.clone()).unwrap();
    directory_index
        .push(PathBuf::from("/non/existent/path"))
        .unwrap();

    // Check if the index file was created
    assert!(index_file_path.exists());

    // Check if the entry was added to the index
    let lines = get_index_file_lines(&index_file_path);
    assert_eq!(lines.len(), 0);
}
