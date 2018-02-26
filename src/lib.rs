extern crate failure;
#[macro_use]
extern crate failure_derive;

pub mod mv_videos {
    #[derive(Debug, Fail)]
    pub enum MvVideosError {
        #[fail(display = "Source directories missing")]
        EmptySources,
        #[fail(display = "Extensions missing")]
        EmptyExtensions,
        #[fail(display = "Invalid size arg '{}'", arg)]
        InvaildSize { arg: String },
        #[fail(display = "Invalid extensions list '{}'", arg)]
        InvalidExtensionsList { arg: String },
    }

    pub fn build_find_cmd(source_dirs: &[&str], min_size: &str, extensions: &[&str]) -> Result<String, MvVideosError> {
        if source_dirs.is_empty() { return Err(MvVideosError::EmptySources); };
        if extensions.is_empty() { return Err(MvVideosError::EmptyExtensions); };

        let srcs = source_dirs
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(" ");
        let exts = extensions
            .iter()
            .map(|s| format!("-name \"*.{}\"", s))
            .collect::<Vec<_>>()
            .join(" -or ");

        Ok(format!("find {} -type f -size +{} {}", srcs, min_size, exts))
    }

    pub fn check_size_arg(size: &str) -> Result<(), MvVideosError> {
        if size.is_empty() { return Err(MvVideosError::InvaildSize { arg: String::from(size) }); };

        let scales: &[_] = &['k', 'M', 'G', 'T', 'P'];
        let last = size.chars().last().unwrap(); // safe because is_empty check
        let size = if scales.contains(&last) {
            size.trim_right_matches(scales)
        } else {
            size
        };

        if let Ok(_) = size.parse::<usize>() {
            return Ok(());
        }

        Err(MvVideosError::InvaildSize { arg: String::from(size) })
    }

    pub fn parse_extensions(ext: &str) -> Result<Vec<&str>, MvVideosError> {
        if ext.is_empty() { return Err(MvVideosError::InvalidExtensionsList { arg: String::from(ext) }); };

        let res: Vec<_> = ext
            .trim_right_matches(',')
            .split(',').collect();

        Ok(res)
    }
}

#[cfg(test)]
extern crate spectral;

#[cfg(test)]
mod test {
    pub use super::*;
    pub use spectral::prelude::*;

    mod build_find {
        use super::*;
        use super::mv_videos::*;

        #[test]
        fn empty_extensions() {
            let res = build_find_cmd(&["one", "two"], "100M", &[]);
            assert_that(&res).is_err();
        }

        #[test]
        fn empty_source_directories() {
            let res = build_find_cmd(&[], "100M", &["avi", "mkv", "mp4"]);
            assert_that(&res).is_err();
        }

        #[test]
        fn find() {
            let res = build_find_cmd(&["one", "two"], "100M", &["avi", "mkv", "mp4"]);
            assert_that(&res)
                .is_ok()
                .is_equal_to(r#"find "one" "two" -type f -size +100M -name "*.avi" -or -name "*.mkv" -or -name "*.mp4""#.to_string());
        }
    }

    mod check_size_arg {
        use super::*;
        use super::mv_videos::*;

        #[test]
        fn empty() {
            let res = check_size_arg("");
            assert_that(&res).is_err();
        }

        #[test]
        fn nan() {
            let res = check_size_arg("a10");
            assert_that(&res).is_err();
        }

        #[test]
        fn bytes() {
            let res = check_size_arg("100");
            assert_that(&res).is_ok();
        }

        #[test]
        fn unknown_scale() {
            let res = check_size_arg("100L");
            assert_that(&res).is_err();
        }

        #[test]
        fn scale_k() {
            let res = check_size_arg("100k");
            assert_that(&res).is_ok();
        }
    }

    mod parse_extension {
        use super::*;
        use super::mv_videos::*;

        #[test]
        fn empty() {
            let res = parse_extensions("");
            assert_that(&res).is_err();
        }

        #[test]
        fn one_extension() {
            let res = parse_extensions("mkv");
            assert_that(&res).is_ok().has_length(1);
        }

        #[test]
        fn two_extension() {
            let res = parse_extensions("mkv,avi");
            assert_that(&res).is_ok().has_length(2);
        }

        #[test]
        fn two_extension_trailing_sep() {
            let res = parse_extensions("mkv,avi,");
            assert_that(&res).is_ok().has_length(2);
        }
    }
}