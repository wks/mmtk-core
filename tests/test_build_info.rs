mod test_build_info {
    #[test]
    fn test_git_version() {
        println!("Git version: {}", *mmtk::build_info::MMTK_GIT_VERSION);
    }

    #[test]
    fn test_full_build_version() {
        println!(
            "Full build version: {}",
            *mmtk::build_info::MMTK_FULL_BUILD_INFO
        );
    }
}
