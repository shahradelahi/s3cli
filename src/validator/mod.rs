use regex::Regex;

pub fn check_profile_name(name: &str) -> bool {
  Regex::new(r"^[a-zA-Z0-9-_]{2,}$").unwrap().is_match(name)
}

#[test]
fn test_check_profile_name() {
  assert!(check_profile_name("test"));
  assert!(check_profile_name("test-123"));
  assert_eq!(false, check_profile_name("test 123"));
}