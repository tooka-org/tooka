use super::file_ops;
use crate::rules::rule::{Action, CopyAction, DeleteAction, MoveAction, RenameAction};
use tempfile::{tempdir, TempDir, NamedTempFile};

fn setup_temp_dir_and_file() -> (TempDir, NamedTempFile) {
    let dir = tempdir().unwrap();
    let src_file = NamedTempFile::new_in(&dir).unwrap();
    (dir, src_file)
}

#[test]
fn test_move_file() {
    let (dir, src_file) = setup_temp_dir_and_file();
    let src_path = src_file.path().to_path_buf();

    let dest_dir = dir.path().join("moved");
    let move_action = Action::Move(MoveAction {
        to: dest_dir.to_str().unwrap().to_string(),
        preserve_structure: false,
    });

    let result = file_ops::execute_action(&src_path, &move_action, false, dir.path()).unwrap();
    assert!(result.new_path.exists());
    assert!(!src_path.exists());
}

#[test]
fn test_copy_file() {
    let (dir, src_file) = setup_temp_dir_and_file();
    let src_path = src_file.path().to_path_buf();

    let dest_dir = dir.path().join("copied");
    let copy_action = Action::Copy(CopyAction {
        to: dest_dir.to_str().unwrap().to_string(),
        preserve_structure: false,
    });

    let result = file_ops::execute_action(&src_path, &copy_action, false, dir.path()).unwrap();
    assert!(result.new_path.exists());
    assert!(src_path.exists());
}

#[test]
fn test_rename_file() {
    let (dir, src_file) = setup_temp_dir_and_file();
    let src_path = src_file.path().to_path_buf();

    let rename_action = Action::Rename(RenameAction {
        to: "renamed_{{ext}}".to_string(),
    });

    let result = file_ops::execute_action(&src_path, &rename_action, false, dir.path()).unwrap();
    assert!(result.new_path.exists());
    assert!(!src_path.exists());
}

#[test]
fn test_delete_file() {
    let (dir, src_file) = setup_temp_dir_and_file();
    let src_path = src_file.path().to_path_buf();

    let delete_action = Action::Delete(DeleteAction { trash: false });

    let result = file_ops::execute_action(&src_path, &delete_action, false, dir.path()).unwrap();
    assert!(!src_path.exists());
    assert_eq!(result.action, "delete");
}
