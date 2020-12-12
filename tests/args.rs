use crate::args::{Args, ArgsError};

#[test]
fn it_works() -> Result<(), ArgsError> {
    let mut resources_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let raw_args = vec!["-f", "a.log"];
    let args = Args::get_with_args(raw_args)?;
    assert_eq!(args.files.len(), 1);
}
