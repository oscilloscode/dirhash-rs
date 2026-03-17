use config::Config;
use std::path::PathBuf;

// TODO: there are other test configs to be added. e.g. how long the random test runs

pub struct TestingFilePaths {
    pub dir: PathBuf,
    pub block_dev: PathBuf,
    pub char_dev: PathBuf,
    pub fifo: PathBuf,
    pub socket: PathBuf,
}

pub fn get_filepath_config() -> TestingFilePaths {
    // TODO: maybe add another source that auto-detects the files?
    // TODO: make it lazy
    let settings = Config::builder()
        .add_source(config::File::with_name("test_config").required(true))
        .add_source(config::Environment::with_prefix("DIRHASH"))
        .build()
        .unwrap();

    TestingFilePaths {
        dir: PathBuf::from(settings.get_string("dir_path").unwrap()),
        block_dev: PathBuf::from(settings.get_string("block_dev_path").unwrap()),
        char_dev: PathBuf::from(settings.get_string("char_dev_path").unwrap()),
        fifo: PathBuf::from(settings.get_string("fifo_path").unwrap()),
        socket: PathBuf::from(settings.get_string("socket_path").unwrap()),
    }
}
