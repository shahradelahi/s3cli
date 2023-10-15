#[derive(Debug)]
pub struct ClarifiedFile {
  path: std::path::PathBuf,
  sha256: String,
  length: usize,
}

impl ClarifiedFile {
  pub fn from_path(path: &str) -> anyhow::Result<Self> {
    if false == std::path::Path::new(&path).exists() {
      return Err(anyhow::anyhow!(format!("File does not exist: {}", &path)));
    }

    let buffer = std::fs::read(&path);
    if let Err(e) = buffer {
      return Err(anyhow::anyhow!(format!("An error occurred while reading file: {}", e.to_string())));
    }
    let buffer = buffer.unwrap();

    let sha256 = sha256::digest(&buffer);

    Ok(Self {
      path: std::path::PathBuf::from(&path),
      sha256,
      length: buffer.len(),
    })
  }
}


#[cfg(test)]
mod clarified_file_tests {
  use super::*;

  #[test]
  fn test_from_path() {
    let file = ClarifiedFile::from_path("LICENSE");
    println!("{:?}", file);
  }
}

/// Returns a list of files on local that matches the given pattern
pub fn list_directory_content(
  path: &str,
  recursive: bool,
) -> anyhow::Result<Vec<ClarifiedFile>> {

  // 1. checking path exists
  if false == std::path::Path::new(&path).exists() {
    return Err(anyhow::anyhow!(format!("Path does not exist: {}", &path)));
  }

  let path = std::path::PathBuf::from(&path);

  // 2. if path was a file return it
  if path.is_file() {
    return ClarifiedFile::from_path(&path.as_path().to_str().unwrap())
       .map(|f| vec![f]);
  }

  // 3. if path was a directory, list all files in it
  let mut files = Vec::new();

  for entry in std::fs::read_dir(path)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_file() {
      files.push(ClarifiedFile::from_path(&path.as_path().to_str().unwrap())?);
    }

    // 4. if recursive, list all files in subdirectories
    if recursive && path.is_dir() {
      // just call this function again
      let mut sub_files = list_directory_content(
        &path.as_path().to_str().unwrap(),
        recursive,
      )?;
      files.append(&mut sub_files);
    }
  }

  Ok(files)
}

#[cfg(test)]
mod list_directory_content_tests {
  use super::*;

  #[test]
  fn test_list_directory_content() {
    let files = list_directory_content(
      "src",
      true,
    );
    assert!(files.is_ok(), "Oh no, it's not ok!");
    for file in files.unwrap() {
      println!("{:?}", file);
    }
  }
}
