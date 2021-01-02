use std::{
    path::{Path, PathBuf},
};

/// get basename with full parent path
fn get_basename(path: &Path) -> PathBuf {
    path.with_file_name(path.file_stem().unwrap())
}

/// path to new asm file
pub fn new_output_asm(path: &Path) -> PathBuf {
    get_basename(path).with_extension("s")
}

/// path to new obj file
pub fn new_output_obj(path: &Path) -> PathBuf {
    get_basename(path).with_extension("o")
}

/// output executable file with using the basename only
pub fn new_output_executable(path: &Path) -> PathBuf {
    get_basename(path)
}

pub fn is_aarch64() -> bool {
    std::env::consts::ARCH == "aarch64"
}

#[derive(Debug, Clone, Copy)]
pub enum TargetOs {
    MacOs,
    Linux,
    Other
}

impl TargetOs {
    pub fn current() -> Self {
        match std::env::consts::OS {
            "macos" => TargetOs::MacOs,
            "linux" => TargetOs::Linux,
            t => TargetOs::Other,
        }
    }
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use test_case::test_case;

    use super::{get_basename, new_output_asm, new_output_obj};

    #[test_case("main.c", "main")]
    #[test_case("../parent_main.c", "../parent_main")]
    #[test_case("tmp/test_long.c", "tmp/test_long")]
    #[test_case("/Users/tmp/test_long.c", "/Users/tmp/test_long")]
    #[test_case("tmp/no_ext", "tmp/no_ext")]
    fn test_basename(src: &str, expected: &str) {
        assert_eq!(
            get_basename(&PathBuf::from(src)).to_str().unwrap(),
            expected
        )
    }

    #[test_case("main.c", "main.s")]
    #[test_case("/Users/tmp/test_long.c", "/Users/tmp/test_long.s")]
    fn test_asm_ouput(src: &str, expected: &str) {
        assert_eq!(
            new_output_asm(&PathBuf::from(src)).to_str().unwrap(),
            expected
        )
    }

    #[test_case("main.c", "main.o")]
    #[test_case("/Users/tmp/test_long.c", "/Users/tmp/test_long.o")]
    fn test_obj_output(src: &str, expected: &str) {
        assert_eq!(
            new_output_obj(&PathBuf::from(src)).to_str().unwrap(),
            expected
        )
    }
}
