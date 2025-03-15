// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analysis;
mod inout;
mod ui_management;

use std::error::Error;
use std::ops::Deref;
use std::rc::Rc;
use rfd::{FileDialog};
use slint::{Model, ModelRc, SharedString, VecModel};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    /*
        GameSaves UI Array
     */
    let gamesaves =  Rc::from(VecModel::default());
    let gamesaves_filepaths = Rc::from(VecModel::default());
    let values = Rc::from(VecModel::default());
    let found_coincidences: i32 = -1;

    ui.set_gamesaves(ModelRc::from(gamesaves.clone()));
    ui.set_gamesaves_paths(ModelRc::from(gamesaves_filepaths.clone()));
    ui.set_values(ModelRc::from(values.clone()));
    ui.set_found_coincidences(found_coincidences.clone());

    /*
        Add Savegame button
     */
    let gamesaves_add = gamesaves.clone();
    let gamesaves_filepaths_add = gamesaves_filepaths.clone();
    let values_add = values.clone();
    ui.on_request_add_savegame({
        move || {

            // Open file explorer
            let file = ui_management::add_gamesave();
            if file.is_some() {
                let file_unwrap = file.unwrap();
                gamesaves_filepaths_add.push(SharedString::from(file_unwrap.0));
                gamesaves_add.push(SharedString::from(file_unwrap.1));
                values_add.push(0);
            };
        }
    });

    /*
        Remove Savegame button
     */
    let gamesaves_remove = gamesaves.clone();
    let gamesaves_filepath_remove = gamesaves_filepaths.clone();
    let values_remove = values.clone();
    ui.on_request_remove_savegame(move |name: SharedString| {
        // find index of name in gamesaves_remove
        let index = gamesaves_remove.iter().position(|x| x == &name).unwrap();
        gamesaves_remove.remove(index);
        gamesaves_filepath_remove.remove(index);
        values_remove.remove(index);
    });

    /*
        let gamesaves_update
     */
    let gamesaves_updatevalue = gamesaves.clone();
    let values_updatevalue = values.clone();
    ui.on_update_savegame_value(move |name: SharedString, val: SharedString| {
        // check for a valid i32
        let val = match val.parse::<i32>() {
            Ok(val) => val,
            _ => 0
        };

        // find index
        let index = gamesaves_updatevalue.iter().position(|x| x == &name).unwrap();
        // modify value
        if index == values_updatevalue.iter().len() { values_updatevalue.push(val) }
        else { values_updatevalue.set_row_data(index, val) }
    });


    /*
        refresh
     */
    let values_refresh = values.clone();
    let gamesaves_filepaths_refresh = gamesaves_filepaths.clone();
    let ui_handle_refresh = ui.as_weak();
    ui.on_request_refresh(move || {
        let ui = ui_handle_refresh.unwrap();

        let mut archivos: Vec<Vec<u8>> = Vec::new();
        for filepath in gamesaves_filepaths_refresh.iter() {
            archivos.push(inout::read_binary_file(filepath.as_str()).unwrap());
        }

        let mut values: Vec<u32> = Vec::new();
        for value in values_refresh.iter() {
            values.push(value as u32);
        }

        println!("archivos: {:?}", archivos);
        println!("valores: {:?}", values);

        let anal = analysis::find_consistent_positions_u32(&archivos, &values);
        let lineas: Vec<u32> = anal.iter().filter_map(|t| Some((((t+1) as f32)/16.0).round() as u32)).collect();

        ui.set_found_coincidences(anal.len() as i32);
        if anal.len() == 1 {
            ui.set_unique_coincidence_index(*anal.get(0).unwrap() as i32);

            // get first file in gamesaves_filepaths_refresh
            let first_file: SharedString = gamesaves_filepaths_refresh.iter().next().unwrap().clone();
            ui.set_gamesave_to_clone_filepath(first_file);
        }

        println!("Líneas con consistencias: {:?}", lineas)
    });

    /*
        modify and save
     */
    let ui_handle_modifynsave = ui.as_weak();
    ui.on_request_modify_n_save(move || {
        let ui = ui_handle_modifynsave.unwrap();

        let new_value = ui.get_selected_modify_value();
        let new_value = new_value.parse::<u32>().unwrap();
        let new_value = new_value.to_le_bytes();

        let modification_index = ui.get_unique_coincidence_index();
        let filename = ui.get_gamesaves().iter().next().unwrap().clone().to_string();
        let file_ext = &[filename.split(".").last().unwrap()];
        let filename = filename.split(".").next().unwrap();

        let file_to_clone_filepath = ui.get_gamesave_to_clone_filepath();
        let file_to_clone_folderpath = file_to_clone_filepath.split("\\").collect::<Vec<&str>>().deref().join("\\");

        let file_to_clone = inout::read_binary_file(&file_to_clone_filepath).unwrap();
        let file_to_clone = inout::modify_file(file_to_clone, modification_index as u32, new_value.to_vec());

        let task = FileDialog::new().add_filter("original format", file_ext)
            .set_directory(file_to_clone_folderpath)
            .set_title("Save modified savegame")
            .set_file_name(filename)
            .save_file();

        match task {
            Some(val) => inout::store_file(file_to_clone, val.to_str().unwrap()).unwrap(),
            _ => println!("Catastrophic error...")
        }

    });

    ui.run()?;

    Ok(())
}

fn convert_from_u32_to_vecu8(value: u32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

fn convert_from_u16_to_vecu8(value: u16) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

// fn asd() {
//     println!("Hello, world!");
//
//     let archivos = vec![
//         inout::read_binary_file("C:\\Users\\srleg\\Desktop\\save21t.csave").unwrap(),
//         inout::read_binary_file("C:\\Users\\srleg\\Desktop\\save20t.csave").unwrap()
//     ];
//
//     let valores: Vec<u32> = vec![21, 20];
//
//     let anal = analysis::find_consistent_positions_u32(&archivos, &valores);
//     let lineas: Vec<u32> = anal.iter().filter_map(|t| Some((((t+1) as f32)/16.0).round() as u32)).collect();
//
//     println!("Líneas con consistencias: {:?}", lineas)
//
// }