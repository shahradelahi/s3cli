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

pub fn is_uuid(str: &String) -> bool {
  Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap().is_match(str)
}

pub fn is_url(str: &String) -> bool {
  Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_+.~#?&/=]*)").unwrap().is_match(str)
}
