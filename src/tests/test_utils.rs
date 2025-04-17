use std::path::PathBuf;

// Finds the root of the project
pub fn find_project_dir() -> PathBuf
{
    let directory = match std::env::current_dir()
    {
        Ok(p) => p,
        Err(_) => PathBuf::new(),
    };

    let mut found_project_dir = PathBuf::new();
    while directory.parent() != None
    {
        let check_path = directory.join("Cargo.toml");
        if check_path.is_file()
        {
            found_project_dir = directory;
            break;
        }
    }

    return found_project_dir;
}

// Creates a full path to an internal file of the project
pub fn create_full_path(p: &str) -> PathBuf
{
    let project_dir = find_project_dir();
    return project_dir.join(p);
}

fn find_all_shaders_in_dir(dir: PathBuf) -> Vec<PathBuf>
{
    let mut paths: Vec<PathBuf> = Vec::new();
    for file_or_directory in std::fs::read_dir(dir).unwrap()
    {
        let path = file_or_directory.unwrap().path();

        if path.is_dir()
        {
            paths.extend(find_all_shaders_in_dir(path))
        }
        else 
        {
            match path.extension()
            {
                None => {},
                Some(ext) => {
                    if ext == "hlsl"
                    {
                        paths.push(path);
                    }
                }
            }
        }
    }

    return paths;
}

// Creates a vector to all shaders
pub fn find_all_shader_paths() -> Vec<PathBuf>
{
    let project_dir = find_project_dir();
    let shaders_dir = project_dir.join("test_files");

    return find_all_shaders_in_dir(shaders_dir);
}