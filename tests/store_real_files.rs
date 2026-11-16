//! Saving on the real file system, in a fresh folder under the system's temp folder.

mod common;

use std::{fs, path::PathBuf};

use tijdje::store::{self, Backup, Overwrite, RealFileSystem};

/// A new, empty folder for one test.
fn scratch_folder(name: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("tijdje-test-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    folder
}

#[test]
fn saving_replaces_the_file_and_keeps_a_backup() {
    let folder = scratch_folder("backup");
    let path = folder.join("timesheet.yaml");
    fs::write(&path, common::SAMPLE).unwrap();

    let (mut timesheet, fingerprint) = store::load(&RealFileSystem, &path).unwrap();
    timesheet.days.clear();
    store::save(
        &RealFileSystem,
        &path,
        &timesheet,
        &fingerprint,
        Overwrite::IfUnchanged,
        Backup::Create,
    )
    .unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "days: {}\n");
    assert_eq!(
        fs::read_to_string(folder.join("timesheet.yaml.bak")).unwrap(),
        common::SAMPLE
    );
    assert!(!folder.join("timesheet.yaml.tmp").exists());
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn saving_a_new_file_creates_it() {
    let folder = scratch_folder("new");
    let path = folder.join("timesheet.yaml");

    let (timesheet, fingerprint) = store::load(&RealFileSystem, &path).unwrap();
    store::save(
        &RealFileSystem,
        &path,
        &timesheet,
        &fingerprint,
        Overwrite::IfUnchanged,
        Backup::Create,
    )
    .unwrap();

    assert!(path.exists());
    assert!(!folder.join("timesheet.yaml.bak").exists());
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn a_file_changed_on_disk_is_not_overwritten_without_permission() {
    let folder = scratch_folder("changed");
    let path = folder.join("timesheet.yaml");
    fs::write(&path, common::SAMPLE).unwrap();

    let (timesheet, fingerprint) = store::load(&RealFileSystem, &path).unwrap();
    fs::write(&path, "days: {}\n").unwrap();
    let result = store::save(
        &RealFileSystem,
        &path,
        &timesheet,
        &fingerprint,
        Overwrite::IfUnchanged,
        Backup::Create,
    );

    assert!(matches!(result, Err(store::SaveError::ChangedOnDisk)));
    assert_eq!(fs::read_to_string(&path).unwrap(), "days: {}\n");
    fs::remove_dir_all(folder).unwrap();
}

#[cfg(unix)]
#[test]
fn saving_through_a_symlink_replaces_the_target_and_keeps_the_symlink() {
    let folder = scratch_folder("symlink");
    let target_folder = folder.join("synced");
    fs::create_dir_all(&target_folder).unwrap();
    let target = target_folder.join("timesheet.yaml");
    fs::write(&target, common::SAMPLE).unwrap();
    let link = folder.join("timesheet.yaml");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let (mut timesheet, fingerprint) = store::load(&RealFileSystem, &link).unwrap();
    timesheet.days.clear();
    store::save(
        &RealFileSystem,
        &link,
        &timesheet,
        &fingerprint,
        Overwrite::IfUnchanged,
        Backup::Create,
    )
    .unwrap();

    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read_to_string(&target).unwrap(), "days: {}\n");
    assert_eq!(
        fs::read_to_string(target_folder.join("timesheet.yaml.bak")).unwrap(),
        common::SAMPLE
    );
    assert!(!folder.join("timesheet.yaml.bak").exists());
    fs::remove_dir_all(folder).unwrap();
}

#[cfg(unix)]
#[test]
fn saving_keeps_the_file_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let folder = scratch_folder("permissions");
    let path = folder.join("timesheet.yaml");
    fs::write(&path, common::SAMPLE).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();

    let (timesheet, fingerprint) = store::load(&RealFileSystem, &path).unwrap();
    store::save(
        &RealFileSystem,
        &path,
        &timesheet,
        &fingerprint,
        Overwrite::IfUnchanged,
        Backup::Create,
    )
    .unwrap();

    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::remove_dir_all(folder).unwrap();
}
