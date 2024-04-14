pub fn examine_dir(dir: &Path, pattern: &str) -> Mutex<HashMap<String, Vec<Line>>> {
    let officials = std::sync::Mutex::new(HashMap::new());
    walk_dir_for_files(dir, pattern,&officials);
    // TODO: fix this shit
    officials
}