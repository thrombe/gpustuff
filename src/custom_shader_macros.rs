
pub fn import_to_string(path: &str) -> String {
    let mut shader = "".to_string();
    let main_mod = std::fs::read_to_string(path).unwrap();
    for line in main_mod.lines() {
        if !line.starts_with("/// ") {
            shader.push_str(line);
            shader.push_str("\n");
            continue;
        }
        let words = line.split_whitespace().collect::<Vec<&str>>();
        match words[1] {
            "import" => {
                let sub_mod = import_to_string(words[2]);
                shader.push_str(&sub_mod);
                shader.push_str("\n");
            },
            _ => (),
        }
    }
    shader
}