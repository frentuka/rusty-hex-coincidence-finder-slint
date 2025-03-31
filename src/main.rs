// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analysis;
mod inout;
mod ui_management;

use std::collections::HashMap;
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
    // (filename, path)
    let gamesave_list: Rc<VecModel<(SharedString, SharedString)>> = Rc::from(VecModel::default());
    let gamesave_values: Rc<VecModel<i32>> = Rc::from(VecModel::default());

    ui.set_gamesave_list(ModelRc::from(gamesave_list.clone()));
    ui.set_gamesave_values(ModelRc::from(gamesave_values.clone()));

    /*
        Coincidence selection stuff
     */
    let found_coincidences_list: Rc<VecModel<(i32, i32)>> = Rc::from(VecModel::default());

    ui.set_checked_for_coincidences(false);
    ui.set_found_coincidences_list(ModelRc::from(found_coincidences_list.clone()));

    /*
        Add Savegame button
     */
    let gamesave_list_add = gamesave_list.clone();
    let gamesave_values_add = gamesave_values.clone();
    ui.on_request_add_savegame({
        move || {
            // Open file explorer
            let file = ui_management::add_gamesave();
            if file.is_some() {
                // 0: path. 1: filename.
                let file_unwrap = file.unwrap();
                let filename = SharedString::from(file_unwrap.0);
                let filepath = SharedString::from(file_unwrap.1);

                gamesave_list_add.push((filename, filepath));
                gamesave_values_add.push(0);
            };
        }
    });

    /*
        Remove Savegame button
     */
    let gamesave_list_remove = gamesave_list.clone();
    let gamesave_values_remove = gamesave_values.clone();
    ui.on_request_remove_savegame(move |name: SharedString| {
        // find index of name in gamesaves_remove
        let index = gamesave_list_remove.iter().position(|x| x.0 == &name).unwrap();
        gamesave_list_remove.remove(index);
        gamesave_values_remove.remove(index);
    });

    /*
        let gamesaves_update
     */
    let gamesave_list_updatevalue = gamesave_list.clone();
    let values_updatevalue = gamesave_values.clone();
    ui.on_update_savegame_value(move |name: SharedString, val: SharedString| {
        // check for a valid i32
        let val = val.parse::<i32>();

        if val.is_ok() {
            let val = val.unwrap();
            // find index
            let index = gamesave_list_updatevalue.iter().position(|x| x.0 == &name).unwrap();
            // modify value
            if index == values_updatevalue.iter().len() { values_updatevalue.push(val) }
            else { values_updatevalue.set_row_data(index, val) }
        }
    });


    /*
        refresh
     */
    let values_refresh = gamesave_values.clone();
    let gamesave_list_refresh = gamesave_list.clone();
    let found_coincidences_list_refresh = found_coincidences_list.clone();
    let ui_handle_refresh = ui.as_weak();
    ui.on_request_refresh(move || {
        let ui = ui_handle_refresh.unwrap();

        let mut archivos: Vec<Vec<u8>> = Vec::new();
        for filepath in gamesave_list_refresh.iter() {
            archivos.push(inout::read_binary_file(filepath.1.as_str()).unwrap());
        }

        let mut values: Vec<u32> = Vec::new();
        for value in values_refresh.iter() {
            values.push(value as u32);
        }

        println!("archivos: {:?}", archivos);
        println!("valores: {:?}", values);

        let anal = analysis::find_consistent_positions_u32(&archivos, &values);
        let lineas: Vec<u32> = anal.iter().filter_map(|t| Some((((t+1) as f32)/16.0).round() as u32)).collect();

        found_coincidences_list_refresh.clear();
        for i in 0..anal.len() {
            found_coincidences_list_refresh.push((
                anal[i] as i32,
                lineas[i] as i32
            ));
        }
        
        println!("Found Coincidences List: {:?}", found_coincidences_list_refresh.iter().size_hint());

        ui.set_checked_for_coincidences(true);
        
        println!("Líneas con consistencias: {:?}", lineas)
    });

    /*
        modify and save
     */
    let ui_handle_modifycoincidence = ui.as_weak();
    ui.on_request_modify_coincidence(move |index: i32| {
        let ui = ui_handle_modifycoincidence.unwrap();

        let new_value = ui.get_selected_modify_value();
        let new_value = new_value.parse::<u32>().unwrap();
        let new_value = new_value.to_le_bytes();
        
        let filename = ui.get_gamesave_list().iter().next().unwrap().clone().0.to_string();
        let file_ext = &[filename.split(".").last().unwrap()];
        let filename = filename.split(".").next().unwrap();

        let file_to_clone_filepath = ui.get_gamesave_list().iter().next().unwrap().clone().1.to_string();
        let file_to_clone_folderpath = file_to_clone_filepath.split("\\").collect::<Vec<&str>>().deref().join("\\");

        let file_to_clone = inout::read_binary_file(&file_to_clone_filepath).unwrap();
        let file_to_clone = inout::modify_file(file_to_clone, index as u32, new_value.to_vec());

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
    //
    // let ui_handle_modifynsave = ui.as_weak();
    // ui.on_request_modify_n_save(move |coincidence: i32| {
    //     let ui = ui_handle_modifynsave.unwrap();
    //
    //     let new_value = ui.get_selected_modify_value();
    //     let new_value = new_value.parse::<u32>().unwrap();
    //     let new_value = new_value.to_le_bytes();
    //
    //     let modification_index = ui.get_selected_coincidence_index();
    //     let filename = ui.get_gamesave_list().iter().next().unwrap().clone().0.to_string();
    //     let file_ext = &[filename.split(".").last().unwrap()];
    //     let filename = filename.split(".").next().unwrap();
    //
    //     let file_to_clone_filepath = ui.get_gamesave_to_clone_filepath();
    //     let file_to_clone_folderpath = file_to_clone_filepath.split("\\").collect::<Vec<&str>>().deref().join("\\");
    //
    //     let file_to_clone = inout::read_binary_file(&file_to_clone_filepath).unwrap();
    //     let file_to_clone = inout::modify_file(file_to_clone, modification_index as u32, new_value.to_vec());
    //
    //     let task = FileDialog::new().add_filter("original format", file_ext)
    //         .set_directory(file_to_clone_folderpath)
    //         .set_title("Save modified savegame")
    //         .set_file_name(filename)
    //         .save_file();
    //
    //     match task {
    //         Some(val) => inout::store_file(file_to_clone, val.to_str().unwrap()).unwrap(),
    //         _ => println!("Catastrophic error...")
    //     }
    //
    // });

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