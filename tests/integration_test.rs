//use std::path::Path;
//use supra::{
//    self,
//    config::{Output, PanConfig, PreConfig, SupraCommand, SupraConfig},
//};

//#[test]
//fn supra_test() {
//    let pre_config = PreConfig::new("./tests/test-input.md", "./tests/test.json", 0, None, false);
//    let pan_config = PanConfig::new(Some("./tests/test-result.md"), None);

//    let config = SupraConfig {
//        command: SupraCommand::Main,
//        output: Some(Output::Markdown),
//        pre_config: Some(pre_config),
//        pan_config: Some(pan_config),
//        post_config: None,
//    };

//    let _ = supra::supra(config);

//    let expected_output = supra::fs::load_file(Path::new("./tests/test-output.md")).unwrap();
//    let result = supra::fs::load_file(Path::new("./tests/test-result.md")).unwrap();

//    assert_eq!(&expected_output.trim(), &result.trim());
//}
