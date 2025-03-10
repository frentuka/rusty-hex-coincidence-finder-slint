pub fn find_pattern_positions(data: &[u8], target: &Vec<u8>) -> Vec<u32> {
    let hex_data: Vec<String> = target.iter().filter_map(|item| Some(format!("{:x}", item))).collect();
    println!("{:?}", hex_data);

    data.windows(target.len()).enumerate().
        filter_map(| (ind, item) |
                       {
                           if item == target { println!("  Found target at index {ind}"); Some(ind as u32) } else { None }
                       })
        .collect()
}

pub fn find_consistent_positions_u32(files: &[Vec<u8>], value: &Vec<u32>) -> Vec<u32> {
    let value: Vec<Vec<u8>> = value.iter().filter_map(|v|
        {
            Some(v.to_le_bytes().to_vec())
        }
    ).collect();
    find_consistent_positions(&files, &value)
}

pub fn find_consistent_positions_u16(files: &[Vec<u8>], value: &Vec<u16>) -> Vec<u32> {
    let value: Vec<Vec<u8>> = value.iter().filter_map(|v|
        {
            Some(v.to_be_bytes().to_vec())
        }
    ).collect();
    find_consistent_positions(&files, &value)
}

pub fn find_consistent_positions(files: &[Vec<u8>], values: &Vec<Vec<u8>>) -> Vec<u32> {
    let mut positions: Vec<u32> = Vec::new();

    for i in 0..files.len() {
        let pattern: Vec<u8> = values[i].to_vec();
        let pos: Vec<u32> = find_pattern_positions(&files[i], &pattern);

        if i == 0 {
            positions = pos; // Tomamos la primera lista de posiciones
        } else {
            positions.retain(|p| pos.contains(p)); // Filtramos solo las posiciones coincidentes
        }
    }

    positions
}