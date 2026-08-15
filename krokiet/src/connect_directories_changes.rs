use std::collections::HashSet;

use rfd::FileDialog;
use slint::{ComponentHandle, Model, ModelRc, VecModel};

use czkawka_core::common::sanitize_path_string;

use crate::{Callabler, ExcludedDirectoriesModel, IncludedDirectoriesModel, MainWindow, Settings};

pub(crate) fn connect_add_remove_directories(app: &MainWindow) {
    connect_add_directories(app);
    connect_remove_directories(app);
    connect_add_manual_directories(app);
}

fn connect_add_manual_directories(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_added_manual_directories(move |included_directories, list_of_files_to_add| {
        let folders = list_of_files_to_add.lines().map(sanitize_path_string).filter(|x| !x.is_empty()).collect::<Vec<_>>();
        if folders.is_empty() {
            return;
        }
        let app = a.upgrade().expect("Failed to upgrade app :(");
        let settings = app.global::<Settings>();

        if included_directories {
            add_included_directories(&settings, &folders);
        } else {
            add_excluded_directories(&settings, &folders);
        }
    });
}

fn filter_model<T: Clone>(model: &ModelRc<T>, index_to_remove: i32) -> Vec<T> {
    model.iter().enumerate().filter(|(idx, _)| *idx as i32 != index_to_remove).map(|(_, item)| item).collect()
}

fn connect_remove_directories(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_remove_item_directories(move |included_directories, index_to_remove| {
        let app = a.upgrade().expect("Failed to upgrade app :(");
        let settings = app.global::<Settings>();

        if included_directories {
            let included_model = settings.get_included_directories_model();
            let new_model = filter_model(&included_model, index_to_remove);

            assert_eq!(included_model.iter().count(), new_model.len() + 1, "Removing item should reduce model size by 1");
            settings.set_included_directories_model(ModelRc::new(VecModel::from(new_model)));
        } else {
            let excluded_model = settings.get_excluded_directories_model();
            let new_model = filter_model(&excluded_model, index_to_remove);

            assert_eq!(excluded_model.iter().count(), new_model.len() + 1, "Removing item should reduce model size by 1");
            settings.set_excluded_directories_model(ModelRc::new(VecModel::from(new_model)));
        }
    });
}

fn connect_add_directories(app: &MainWindow) {
    let a = app.as_weak();
    app.on_folder_choose_requested(move |included_directories| {
        let app = a.upgrade().expect("Failed to upgrade app :(");

        let directory = std::env::current_dir().unwrap_or(std::path::PathBuf::from("/"));

        let file_dialog = FileDialog::new().set_directory(directory);

        let Some(folders) = file_dialog.pick_folders() else {
            return;
        };
        let folders = folders.iter().map(|x| x.to_string_lossy().to_string()).collect::<Vec<_>>();

        let settings = app.global::<Settings>();
        if included_directories {
            add_included_directories(&settings, &folders);
        } else {
            add_excluded_directories(&settings, &folders);
        }
    });
}

fn new_directory_paths(existing_paths: impl IntoIterator<Item = String>, folders: &[String]) -> Vec<String> {
    let mut seen_paths = existing_paths.into_iter().collect::<HashSet<_>>();
    folders.iter().filter(|path| seen_paths.insert((*path).clone())).cloned().collect()
}

fn add_included_directories(settings: &Settings, folders: &[String]) {
    let old_folders = settings.get_included_directories_model();
    let mut new_folders = old_folders.iter().collect::<Vec<_>>();
    let filtered_folders = new_directory_paths(old_folders.iter().map(|folder| folder.path.to_string()), folders);

    for x in &mut new_folders {
        x.selected_row = false;
    }

    new_folders.extend(filtered_folders.into_iter().map(|path| IncludedDirectoriesModel {
        path: path.into(),
        referenced_folder: false,
        selected_row: false,
    }));

    new_folders.sort_by_key(|x| x.path.clone());

    let new_folders_model = ModelRc::new(VecModel::from(new_folders));
    settings.set_included_directories_model(new_folders_model);
}

fn add_excluded_directories(settings: &Settings, folders: &[String]) {
    let old_folders = settings.get_excluded_directories_model();
    let mut new_folders = old_folders.iter().collect::<Vec<_>>();
    let filtered_folders = new_directory_paths(old_folders.iter().map(|folder| folder.path.to_string()), folders);

    for x in &mut new_folders {
        x.selected_row = false;
    }

    new_folders.extend(filtered_folders.into_iter().map(|path| ExcludedDirectoriesModel {
        path: path.into(),
        selected_row: false,
    }));

    new_folders.sort_by_key(|x| x.path.clone());

    let new_folders_model = ModelRc::new(VecModel::from(new_folders));
    settings.set_excluded_directories_model(new_folders_model);
}

#[cfg(test)]
mod tests {
    use super::new_directory_paths;

    #[test]
    fn new_directory_paths_skips_existing_and_repeated_entries() {
        let existing_paths = vec!["C:\\existing".to_string()];
        let folders = vec![
            "C:\\existing".to_string(),
            "C:\\new".to_string(),
            "C:\\new".to_string(),
            "C:\\other".to_string(),
            "C:\\other".to_string(),
        ];

        assert_eq!(new_directory_paths(existing_paths, &folders), vec!["C:\\new", "C:\\other"]);
    }
}
