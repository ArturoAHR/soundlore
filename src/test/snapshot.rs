use std::{
    collections::HashSet,
    sync::{LazyLock, Mutex},
};

use iced_test::simulator::Snapshot;

pub const SNAPSHOTS_DIRECTORY: &str = "snapshots/";
pub const SNAPSHOTS_IMAGES_DIRECTORY: &str = "snapshots/images/";

pub static SNAPSHOT_NAME_SET: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

fn get_snapshot_name(snapshot_name: String) -> String {
    let mut snapshot_name = snapshot_name;

    let mut snapshot_names_used = SNAPSHOT_NAME_SET.lock().unwrap();

    if snapshot_names_used.contains(&snapshot_name) {
        let mut snapshot_name_index = 0;
        snapshot_name = loop {
            snapshot_name_index += 1;
            let snapshot_name = format!("{snapshot_name}_{snapshot_name_index}");

            if snapshot_names_used.contains(&snapshot_name) {
                continue;
            }

            break snapshot_name;
        };
    }

    snapshot_names_used.insert(snapshot_name.clone());

    snapshot_name
}

pub fn assert_snapshot(snapshot: &Snapshot) {
    let snapshot_name = get_snapshot_name(
        std::thread::current()
            .name()
            .unwrap_or("unnamed")
            .replace("::", "__"),
    );

    match_snapshot(snapshot, &snapshot_name);
}

pub fn assert_snapshot_with_name(snapshot: &Snapshot, snapshot_name: &str) {
    let snapshot_name = get_snapshot_name(format!(
        "{}_{snapshot_name}",
        std::thread::current()
            .name()
            .unwrap_or("unnamed")
            .replace("::", "__")
    ));

    match_snapshot(snapshot, &snapshot_name);
}

fn match_snapshot(snapshot: &Snapshot, snapshot_name: &str) {
    let snapshot_image_file_path = format!("{SNAPSHOTS_IMAGES_DIRECTORY}/{snapshot_name}");
    let snapshot_hash_file_path = format!("{SNAPSHOTS_DIRECTORY}/{snapshot_name}");

    let _ = std::fs::remove_file(format!("{snapshot_image_file_path}-tiny-skia.png"));
    let _ = snapshot.matches_image(&snapshot_image_file_path);

    let snapshot_match_result = snapshot.matches_hash(&snapshot_hash_file_path);

    match snapshot_match_result {
        Ok(true) => {}
        Ok(false) => panic!("Snapshot does not match snapshot \"{snapshot_name}\""),
        Err(error) => panic!("Snapshot matching error: {error}"),
    }
}
