use std::fs;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tempfile::TempDir;

use crate::common::model::{CheckingMethod, HashType};
use crate::common::tool_data::{CommonData, DeleteMethod};
use crate::common::traits::Search;
use crate::tools::duplicate::{DuplicateFinder, DuplicateFinderParameters};

#[test]
fn test_find_duplicates_by_hash() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create duplicate files with same content
    fs::write(path.join("file1.txt"), b"duplicate content").unwrap();
    fs::write(path.join("file2.txt"), b"duplicate content").unwrap();
    fs::write(path.join("unique.txt"), b"unique content").unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::Hash, HashType::Blake3, false, false, 0, 0, true);

    let mut finder = DuplicateFinder::new(params);
    finder.set_included_directory(vec![path.to_path_buf()]);
    finder.set_minimal_file_size(0);
    finder.set_recursive_search(true);
    finder.set_use_cache(false);

    let stop_flag = Arc::new(AtomicBool::new(false));
    finder.search(&stop_flag, None);

    let info = finder.get_information();
    assert_eq!(info.number_of_groups_by_hash, 1, "Should find 1 group of duplicates");
    assert_eq!(info.number_of_duplicated_files_by_hash, 1, "Should find 1 duplicate file");
}

#[test]
fn test_find_duplicates_by_size_name_and_hash() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();
    for directory in ["first", "second", "third", "fourth"] {
        fs::create_dir(path.join(directory)).unwrap();
    }

    fs::write(path.join("first").join("same.txt"), b"matching").unwrap();
    fs::write(path.join("second").join("same.txt"), b"matching").unwrap();
    fs::write(path.join("third").join("other.txt"), b"matching").unwrap();
    fs::write(path.join("fourth").join("same.txt"), b"mismatch").unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::SizeNameHash, HashType::Blake3, false, false, 0, 0, true);
    let mut finder = DuplicateFinder::new(params);
    finder.set_included_directory(vec![path.to_path_buf()]);
    finder.set_minimal_file_size(0);
    finder.set_use_cache(false);

    finder.search(&Arc::new(AtomicBool::new(false)), None);

    let info = finder.get_information();
    assert_eq!(info.number_of_groups_by_hash, 1);
    assert_eq!(info.number_of_duplicated_files_by_hash, 1);
    let duplicate_group = finder.get_files_sorted_by_hash().values().flatten().next().unwrap();
    assert_eq!(duplicate_group.len(), 2);
    assert!(duplicate_group.iter().all(|entry| entry.path.file_name().unwrap() == "same.txt"));
}

#[test]
fn test_size_name_hash_case_insensitive_name_comparison() {
    let temp_dir = TempDir::new().unwrap();
    let first_directory = temp_dir.path().join("first");
    let second_directory = temp_dir.path().join("second");
    fs::create_dir(&first_directory).unwrap();
    fs::create_dir(&second_directory).unwrap();
    fs::write(first_directory.join("SAME.txt"), b"matching").unwrap();
    fs::write(second_directory.join("same.txt"), b"matching").unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::SizeNameHash, HashType::Blake3, false, false, 0, 0, false);
    let mut finder = DuplicateFinder::new(params);
    finder.set_included_directory(vec![temp_dir.path().to_path_buf()]);
    finder.set_minimal_file_size(0);
    finder.set_use_cache(false);

    finder.search(&Arc::new(AtomicBool::new(false)), None);

    let info = finder.get_information();
    assert_eq!(info.number_of_groups_by_hash, 1);
    assert_eq!(info.number_of_duplicated_files_by_hash, 1);
}

#[test]
fn test_size_name_hash_deletes_only_non_reference_files() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();
    let reference_directory = path.join("reference");
    let first_directory = path.join("first");
    let second_directory = path.join("second");
    for directory in [&reference_directory, &first_directory, &second_directory] {
        fs::create_dir(directory).unwrap();
        fs::write(directory.join("same.txt"), b"matching").unwrap();
    }

    let params = DuplicateFinderParameters::new(CheckingMethod::SizeNameHash, HashType::Blake3, false, false, 0, 0, true);
    let mut finder = DuplicateFinder::new(params);
    finder.set_included_directory(vec![path.to_path_buf()]);
    finder.set_reference_directory(vec![reference_directory.clone()]);
    finder.set_minimal_file_size(0);
    finder.set_use_cache(false);
    finder.set_delete_method(DeleteMethod::AllExceptNewest);
    finder.set_dry_run(true);

    finder.search(&Arc::new(AtomicBool::new(false)), None);

    let messages = &finder.get_text_messages().messages;
    let messages_lowercase = messages.iter().map(|message| message.to_lowercase()).collect::<Vec<_>>();
    assert_eq!(messages.len(), 2);
    assert!(messages_lowercase.iter().any(|message| message.contains(&first_directory.to_string_lossy().to_lowercase())));
    assert!(
        messages_lowercase
            .iter()
            .any(|message| message.contains(&second_directory.to_string_lossy().to_lowercase()))
    );
    assert!(
        messages_lowercase
            .iter()
            .all(|message| !message.contains(&reference_directory.to_string_lossy().to_lowercase()))
    );
}

#[test]
fn test_size_name_hash_hardlink_preview_uses_reference_source() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();
    let reference_directory = path.join("reference");
    let normal_directory = path.join("normal");
    for directory in [&reference_directory, &normal_directory] {
        fs::create_dir(directory).unwrap();
        fs::write(directory.join("same.txt"), b"matching").unwrap();
    }

    let params = DuplicateFinderParameters::new(CheckingMethod::SizeNameHash, HashType::Blake3, false, false, 0, 0, true);
    let mut finder = DuplicateFinder::new(params);
    finder.set_included_directory(vec![path.to_path_buf()]);
    finder.set_reference_directory(vec![reference_directory.clone()]);
    finder.set_minimal_file_size(0);
    finder.set_use_cache(false);
    finder.set_delete_method(DeleteMethod::HardLink);
    finder.set_dry_run(true);

    finder.search(&Arc::new(AtomicBool::new(false)), None);

    let messages = &finder.get_text_messages().messages;
    assert_eq!(messages.len(), 1);
    let message = messages[0].to_lowercase();
    assert!(message.contains(&reference_directory.to_string_lossy().to_lowercase()));
    assert!(message.contains(&normal_directory.to_string_lossy().to_lowercase()));
}

#[test]
fn test_find_duplicates_by_size() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create files with same size
    fs::write(path.join("file1.txt"), b"12345").unwrap();
    fs::write(path.join("file2.txt"), b"abcde").unwrap();
    fs::write(path.join("unique.txt"), b"123").unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::Size, HashType::Blake3, false, false, 0, 0, true);

    let mut finder = DuplicateFinder::new(params);
    finder.set_included_directory(vec![path.to_path_buf()]);
    finder.set_recursive_search(true);
    finder.set_minimal_file_size(0);
    finder.set_use_cache(false);

    let stop_flag = Arc::new(AtomicBool::new(false));
    finder.search(&stop_flag, None);

    let info = finder.get_information();
    assert_eq!(info.number_of_groups_by_size, 1, "Should find 1 group by size");
    assert_eq!(info.number_of_duplicated_files_by_size, 1, "Should find 1 duplicate by size");
}

#[test]
fn test_find_duplicates_by_name() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    let dir1 = path.join("dir1");
    let dir2 = path.join("dir2");
    fs::create_dir(&dir1).unwrap();
    fs::create_dir(&dir2).unwrap();

    // Create files with same name in different directories
    fs::write(dir1.join("duplicate.txt"), b"content1").unwrap();
    fs::write(dir2.join("duplicate.txt"), b"content2").unwrap();
    fs::write(dir1.join("unique.txt"), b"unique").unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::Name, HashType::Blake3, false, false, 0, 0, true);

    let mut finder = DuplicateFinder::new(params);
    finder.set_recursive_search(true);
    finder.set_included_directory(vec![path.to_path_buf()]);
    finder.set_minimal_file_size(0);
    finder.set_use_cache(false);
    let stop_flag = Arc::new(AtomicBool::new(false));
    finder.search(&stop_flag, None);

    let info = finder.get_information();
    assert_eq!(info.number_of_groups_by_name, 1, "Should find 1 group by name");
    assert_eq!(info.number_of_duplicated_files_by_name, 1, "Should find 1 duplicate by name");
}

#[test]
fn test_case_insensitive_name_comparison() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();
    let first_directory = path.join("first");
    let second_directory = path.join("second");
    fs::create_dir(&first_directory).unwrap();
    fs::create_dir(&second_directory).unwrap();

    // Different directories keep these distinct on case-insensitive file systems.
    fs::write(first_directory.join("TEST.txt"), b"content1").unwrap();
    fs::write(second_directory.join("test.txt"), b"content2").unwrap();

    let params = DuplicateFinderParameters::new(
        CheckingMethod::Name,
        HashType::Blake3,
        false,
        false,
        0,
        0,
        false, // case insensitive
    );

    let mut finder = DuplicateFinder::new(params);
    finder.set_recursive_search(true);
    finder.set_use_cache(false);
    finder.set_minimal_file_size(0);
    finder.set_included_directory(vec![path.to_path_buf()]);

    let stop_flag = Arc::new(AtomicBool::new(false));
    finder.search(&stop_flag, None);
    let info = finder.get_information();
    assert_eq!(info.number_of_groups_by_name, 1, "Should find duplicates with case insensitive search");
}

#[test]
fn test_no_duplicates_found() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create unique files
    fs::write(path.join("file1.txt"), b"content1").unwrap();
    fs::write(path.join("file2.txt"), b"content2").unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::Hash, HashType::Blake3, false, false, 0, 0, true);

    let mut finder = DuplicateFinder::new(params);
    finder.set_included_directory(vec![path.to_path_buf()]);
    finder.set_recursive_search(true);
    finder.set_use_cache(false);
    finder.set_minimal_file_size(0);

    let stop_flag = Arc::new(AtomicBool::new(false));
    finder.search(&stop_flag, None);

    let info = finder.get_information();
    assert_eq!(info.number_of_groups_by_hash, 0, "Should find no duplicate groups");
    assert_eq!(info.lost_space_by_hash, 0, "Should have no lost space");
}

#[test]
fn test_lost_space_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create 3 files with 100 bytes each, all duplicates
    let content = vec![b'A'; 100];
    fs::write(path.join("file1.txt"), &content).unwrap();
    fs::write(path.join("file2.txt"), &content).unwrap();
    fs::write(path.join("file3.txt"), &content).unwrap();

    let params = DuplicateFinderParameters::new(CheckingMethod::Hash, HashType::Blake3, false, false, 0, 0, true);

    let mut finder = DuplicateFinder::new(params);
    finder.set_minimal_file_size(0);
    finder.set_use_cache(false);
    finder.set_included_directory(vec![path.to_path_buf()]);

    let stop_flag = Arc::new(AtomicBool::new(false));
    finder.search(&stop_flag, None);

    let info = finder.get_information();
    assert_eq!(info.lost_space_by_hash, 200, "Should calculate 200 bytes lost space (2 duplicate files * 100 bytes)");
}
